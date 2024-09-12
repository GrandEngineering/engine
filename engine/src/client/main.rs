use enginelib;
use proto::engine_client::EngineClient;
use std::error::Error;
pub mod proto {
    tonic::include_proto!("engine");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";
    let mut client = EngineClient::connect(url).await?;
    let req = proto::TaskRequest {
        task_type: proto::TaskType::TaskFib.into(),
    };
    let res = client.aquire_task(req).await?;'
    println!("{:?}", res);
    Ok(())
}
