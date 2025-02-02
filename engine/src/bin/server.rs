use bincode::serialize;
use enginelib::{
    api::EngineAPI,
    events::{self, Events, ID},
    plugin::LibraryManager,
    task::{TaskQueue, TaskQueueStorage},
    Identifier, RawIdentier, Registry, StartEvent,
};
use proto::engine_server::{Engine, EngineServer};
use std::sync::Arc;
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[allow(non_snake_case)]
struct EngineService {
    pub EngineAPI: EngineAPI,
    pub libs: LibraryManager,
    pub db: sled::Db,
}
#[tonic::async_trait]
impl Engine for EngineService {
    async fn aquire_task_reg(
        &self,
        request: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::TaskRegistry>, tonic::Status> {
        let mut tasks: Vec<RawIdentier> = Vec::new();
        for (k, v) in &self.EngineAPI.task_registry.tasks {
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
        let task_id = String::from_utf8(input.task_id.clone()).unwrap();

        let namespace = &task_id.split(":").collect::<Vec<&str>>()[0];
        let task_name = &task_id.split(":").collect::<Vec<&str>>()[1];
        println!("namespace:task {}:{}", &namespace, &task_name);
        let response = proto::Task {
            ..Default::default()
        };
        Ok(tonic::Response::new(response))
    }
    async fn create_task(
        &self,
        request: tonic::Request<proto::Task>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        Err(tonic::Status::aborted("Error"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = EngineAPI::default();
    Events::init(&mut api);
    let mut lib_manager = LibraryManager::default();
    lib_manager.load_library("target/debug/libengine_core.so", &mut api);
    StartEvent!(lib_manager, api);
    let addr = "[::1]:50051".parse().unwrap();
    let db: sled::Db = sled::open("engine_db")?;
    let task_queue = TaskQueueStorage::default();
    let te = bincode::serialize(&task_queue).unwrap();
    db.insert("tasks", te)?;
    let engine = EngineService {
        EngineAPI: api,
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
