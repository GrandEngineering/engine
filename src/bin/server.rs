use bincode::serialize;
use engine::{get_auth, get_uid};
use enginelib::{
    Identifier, RawIdentier, Registry,
    api::EngineAPI,
    chrono::Utc,
    event::{debug, info},
    events::{self, Events, ID},
    plugin::LibraryManager,
    task::{Task, TaskQueue},
};
use proto::engine_server::{Engine, EngineServer};
use std::{
    env::consts::OS,
    io::Read,
    sync::{Arc, RwLock as RS_RwLock},
};
use tokio::sync::RwLock;
use tonic::{Request, Status, metadata::MetadataValue, transport::Server};

mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[allow(non_snake_case)]
struct EngineService {
    pub EngineAPI: Arc<RwLock<EngineAPI>>,
}
#[tonic::async_trait]
impl Engine for EngineService {
    async fn cgrpc(
        &self,
        request: tonic::Request<proto::Cgrpcmsg>,
    ) -> std::result::Result<tonic::Response<proto::Cgrpcmsg>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let db = api.db.clone();
        let output = Events::CheckAdminAuth(
            &mut api,
            challenge,
            (
                request.get_ref().handler_mod_id.clone(),
                request.get_ref().handler_id.clone(),
            ),
            db,
        );
        if !output {
            return Err(tonic::Status::permission_denied("Invalid CGRPC Auth"));
        };
        let mut out = Arc::new(std::sync::RwLock::new(Vec::new()));
        Events::CgrpcEvent(
            &mut api,
            ID("engine_core", "grpc"),
            request.get_ref().event_payload.clone(),
            out.clone(),
        );
        let mut res = request.get_ref().clone();
        res.event_payload = out.read().unwrap().clone();
        return Ok(tonic::Response::new(res));
    }
    async fn aquire_task_reg(
        &self,
        request: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::TaskRegistry>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let db = api.db.clone();
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        if !Events::CheckAuth(&mut api, uid, challenge, db) {
            info!("Aquire Task Reg denied due to Invalid Auth");
            return Err(Status::permission_denied("invalid auth"));
        };
        let mut tasks: Vec<RawIdentier> = Vec::new();
        for (k, v) in &api.task_registry.tasks {
            let js: Vec<String> = vec![k.0.clone(), k.1.clone()];
            let jstr = js.join(":");
            tasks.push(jstr);
        }
        let response = proto::TaskRegistry { tasks };
        Ok(tonic::Response::new(response))
    }

    async fn aquire_task(
        &self,
        request: tonic::Request<proto::TaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();
        if !Events::CheckAuth(&mut api, uid.clone(), challenge, db) {
            info!("Aquire Task denied due to Invalid Auth");
            return Err(Status::permission_denied("invalid auth"));
        };

        // Todo: check for wrong input to not cause a Panic out of bounds.
        let input = request.get_ref();
        let task_id = input.task_id.clone();
        let alen = &task_id.split(":").collect::<Vec<&str>>().len();
        if *alen != 2 {
            return Err(Status::invalid_argument("Invalid Params"));
        }
        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];
        let tsx = api
            .task_registry
            .get(&(namespace.to_string(), task_name.to_string()));
        if tsx.is_none() {
            return Err(Status::invalid_argument("Task Does not Exist"));
        }
        let mut map = api
            .task_queue
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        let ttask = map.first().unwrap().clone();
        let task_payload = ttask.bytes.clone();
        map.remove(0);
        // Get Task and remove it from queue
        api.task_queue.tasks.insert(ID(namespace, task_name), map);
        let store = bincode::serialize(&api.task_queue.clone()).unwrap();
        api.db.insert("tasks", store).unwrap();
        // Move it to exec queue
        let mut exec_tsks = api
            .executing_tasks
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        exec_tsks.push(enginelib::task::StoredExecutingTask {
            bytes: task_payload.clone(),
            user_id: uid.clone(),
            given_at: Utc::now(),
            id: ttask.id.clone(),
        });
        api.executing_tasks
            .tasks
            .insert(ID(namespace, task_name), exec_tsks);
        let store = bincode::serialize(&api.executing_tasks.clone()).unwrap();
        api.db.insert("executing_tasks", store).unwrap();
        let response = proto::Task {
            id: ttask.id,
            task_id: input.task_id.clone(),
            task_payload,
            payload: Vec::new(),
        };
        Ok(tonic::Response::new(response))
    }
    async fn publish_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Empty>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();

        let task_id = request.get_ref().task_id.clone();
        let alen = &task_id.split(":").collect::<Vec<&str>>().len();
        if *alen != 2 {
            return Err(Status::invalid_argument("Invalid Params"));
        }
        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];

        if !Events::CheckAuth(&mut api, uid.clone(), challenge, db) {
            info!("Aquire Task denied due to Invalid Auth");
            return Err(Status::permission_denied("invalid auth"));
        };
        let mem_tsk = api
            .executing_tasks
            .tasks
            .get(&ID(namespace, task_name))
            .unwrap()
            .clone();
        let tsk = mem_tsk
            .iter()
            .find(|f| f.id == task_id.clone() && f.user_id == uid.clone());
        if let Some(tsk) = tsk {
            // Exec Tasks -> DB
            let mut nmem_tsk = mem_tsk.clone();
            nmem_tsk.retain(|f| f.id != task_id.clone() && f.user_id != uid.clone());
            api.executing_tasks
                .tasks
                .insert(ID(namespace, task_name), nmem_tsk.clone());
            let t_mem_execs = api.executing_tasks.clone();
            api.db
                .insert("executing_tasks", bincode::serialize(&t_mem_execs).unwrap())
                .unwrap();
            // tsk-> solved Tsks
            let mut mem_solv = api
                .solved_tasks
                .tasks
                .get(&ID(namespace, task_name))
                .unwrap()
                .clone();
            mem_solv.push(enginelib::task::StoredTask {
                bytes: tsk.bytes.clone(),
                id: tsk.id.clone(),
            });
            api.solved_tasks
                .tasks
                .insert(ID(namespace, task_name), mem_solv);
            // Solved tsks -> DB
            let e_solv = bincode::serialize(&api.solved_tasks.tasks).unwrap();
            api.db.insert("solved_tasks", e_solv).unwrap();
        } else {
            return Err(tonic::Status::not_found("Invalid taskid or userid"));
        }
        Err(tonic::Status::ok("message"))
    }
    async fn create_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
        let challenge = get_auth(&request);
        let uid = get_uid(&request);
        let db = api.db.clone();
        if !Events::CheckAuth(&mut api, uid, challenge, db) {
            info!("Create Task denied due to Invalid Auth");
            return Err(Status::permission_denied("invalid auth"));
        };
        let task = request.get_ref();
        let task_id = task.task_id.clone();
        let id: Identifier = (
            task_id.split(":").collect::<Vec<&str>>()[0].to_string(),
            task_id.split(":").collect::<Vec<&str>>()[1].to_string(),
        );
        let tsk_inst = self.EngineAPI.read().await.task_registry.get(&id).unwrap();
        let tsk: Box<dyn Task> = tsk_inst.from_bytes(&task.task_payload);
        // self.EngineAPI
        //     .write()
        //     .await
        //     .task_queue
        //     .tasks
        //     .get(&id)
        //     .unwrap()
        //     .lock()
        //     .unwrap()
        //     .push(tsk);
        Err(tonic::Status::aborted("Error"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = EngineAPI::default();
    EngineAPI::init(&mut api);
    Events::init_auth(&mut api);
    Events::StartEvent(&mut api);
    let addr = api.cfg.config_toml.port.parse().unwrap();
    let apii = Arc::new(RwLock::new(api));
    EngineAPI::init_chron(apii.clone());
    let engine = EngineService { EngineAPI: apii };

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap();

    Server::builder()
        .add_service(reflection_service)
        .add_service(EngineServer::new(engine))
        .serve(addr)
        .await?;

    Ok(())
}
