use proto::engine_client;
//use enginelib::EventHandler;

use std::error::Error;

pub mod proto {
    tonic::include_proto!("engine");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";
    let mut client = engine_client::EngineClient::connect(url).await?;

    let req = proto::Empty {};
    let request = tonic::Request::new(req);
    let response = client.aquire_task_reg(request).await?;
    let vec = response.get_ref().tasks.clone();
    Ok(())
}
