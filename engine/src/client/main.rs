use enginelib::*;
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
    let request = tonic::Request::new(req);
    let res = client.aquire_task(request).await?;
    let data = res.get_ref();
    let mut decoded: FibTask = enginelib::FibTask::from_bytes(data.task_payload.as_slice());
    decoded.run(Some(Runner::CPU));
    println!("{:?}", decoded);
    Ok(())
}
