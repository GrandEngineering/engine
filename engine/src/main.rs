use proto::engine_server::{Engine, EngineServer};

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
        println!("Got a request {:?}", request);
        let input = request.get_ref();
        let task = proto::Task {
            id: "pog".to_string(),
            runner: input.runner,
            task: input.task,
            task_data: Option::from(proto::task::TaskData::Aksprime(proto::AksprimeData {
                start: Vec::new(),
                end: Vec::new(),
            })),
        };
        Ok(tonic::Response::new(task))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let engine = EngineService::default();
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(reflection_service)
        .add_service(EngineServer::new(engine))
        .serve(addr)
        .await?;
    Ok(())
}
