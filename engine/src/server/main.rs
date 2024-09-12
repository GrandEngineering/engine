use enginelib::{FibTask, Task};
use proto::engine_server::{Engine, EngineServer};
use proto::TaskType;
use tonic::transport::Server;
mod proto {
    tonic::include_proto!("engine");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("engine_descriptor");
}
#[derive(Debug, Default)]
struct EngineService {}

#[tonic::async_trait]
impl Engine for EngineService {
    async fn aquire_task(
        &self,
        request: tonic::Request<proto::TaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let input = request.get_ref();
        println!("Got a request {:?}", input);
        let payload = FibTask {
            iter: 10,
            result: 0,
        };
        let task = proto::Task {
            task_payload: payload.to_bytes(),
            task_type: 0,
        };
        println!("{:?}", task);
        Ok(tonic::Response::new(task))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
