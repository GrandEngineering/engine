use bincode::serialize;
use enginelib::{
    Identifier, RawIdentier, Registry,
    api::EngineAPI,
    event::debug,
    events::{self, Events, ID},
    plugin::LibraryManager,
    task::{Task, TaskQueue, TaskQueueStorage},
};
use proto::engine_server::{Engine, EngineServer};
use std::{env::consts::OS, sync::Arc};
use tokio::sync::RwLock;
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[allow(non_snake_case)]
struct EngineService {
    pub EngineAPI: Arc<RwLock<EngineAPI>>,
    pub libs: LibraryManager,
    pub db: sled::Db,
}
#[tonic::async_trait]
impl Engine for EngineService {
    async fn cgrpc(
        &self,
        request: tonic::Request<proto::Cgrpcmsg>,
    ) -> std::result::Result<tonic::Response<proto::Cgrpcmsg>, tonic::Status> {
        let mut api = self.EngineAPI.write().await;
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
        let mut tasks: Vec<RawIdentier> = Vec::new();
        for (k, v) in &self.EngineAPI.read().await.task_registry.tasks {
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
        // Todo: check for wrong input to not cause a Panic out of bounds.
        let input = request.get_ref();
        println!("Got a request {:?}", input);
        let task_id = input.task_id.clone();

        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];
        println!("namespace:task {}:{}", &namespace, &task_name);
        let tsx = self
            .EngineAPI
            .read()
            .await
            .task_registry
            .get(&(namespace.to_string(), task_name.to_string()))
            .unwrap();
        let response = proto::Task {
            task_id: input.task_id.clone(),
            task_payload: tsx.to_bytes(),
        };
        Ok(tonic::Response::new(response))
    }
    async fn create_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let task = request.get_ref();
        let task_id = task.task_id.clone();
        let id: Identifier = (
            task_id.split(":").collect::<Vec<&str>>()[0].to_string(),
            task_id.split(":").collect::<Vec<&str>>()[1].to_string(),
        );
        let tsk_inst = self.EngineAPI.read().await.task_registry.get(&id).unwrap();
        let tsk: Box<dyn Task> = tsk_inst.from_bytes(&task.task_payload);
        self.EngineAPI.write().await.task_queue.tasks.push(tsk);
        Err(tonic::Status::aborted("Error"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = EngineAPI::default();
    EngineAPI::setup_logger();
    Events::init(&mut api);
    let mut lib_manager = LibraryManager::default();
    #[cfg(feature = "dev")]
    lib_manager.load_library("./target/release/libengine_core.so", &mut api);
    #[cfg(not(feature = "dev"))]
    lib_manager.load_modules(&mut api);
    Events::StartEvent(&mut api, &mut lib_manager);
    let addr = "[::1]:50051".parse().unwrap();
    let db: sled::Db = sled::open("engine_db")?;
    let task_queue = TaskQueueStorage::default();
    let te = bincode::serialize(&task_queue).unwrap();
    db.insert("tasks", te)?;
    let engine = EngineService {
        EngineAPI: Arc::new(RwLock::new(api)),
        libs: lib_manager,
        db,
    };

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
