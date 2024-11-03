use enginelib::event::{EngineAPI, OnStartEvent};
use enginelib::{event, Registry};
use proto::engine_server::{Engine, EngineServer};
use std::sync::Arc;
use tonic::transport::Server;

#[cfg(unix)]
use libloading::os::unix::*;

#[cfg(windows)]
use libloading::os::windows::*;

mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[derive(Debug, Default)]
struct EngineService {}

#[tonic::async_trait]
impl Engine for EngineService {
    async fn aquire_task_reg(
        &self,
        request: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::TaskRegistry>, tonic::Status> {
        let response = proto::TaskRegistry { tasks: vec![] };
        Ok(tonic::Response::new(response))
    }
    async fn aquire_task(
        &self,
        request: tonic::Request<proto::TaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = event::EngineAPI::default();
    let start_event = ("core".to_string(), "onstartevent".to_string());
    unsafe {
        let lib = Library::new("modules/libengine_core.so").unwrap();
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)> = lib.get(b"run").unwrap();
        run(&mut api);
    }
    println!(
        "BIN:{:?}",
        api.task_registry
            .tasks
            .get(&("namespace".to_string(), "fib".to_string()))
    );
    api.event_bus.event_registry.register(
        //with any valid state
        Arc::new(event::OnStartEvent {
            cancelled: false,
            modules: vec![],
            id: start_event.clone(),
        }),
        start_event.clone(),
    );
    api.event_bus.handle(
        start_event.clone(),
        &mut OnStartEvent {
            cancelled: false,
            id: start_event.clone(),
            modules: vec![],
        },
    );
    let addr = "[::1]:50051".parse().unwrap();
    let engine = EngineService::default();
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
