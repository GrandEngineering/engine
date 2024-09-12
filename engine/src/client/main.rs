use proto::engine_client::EngineClient;
use std::error::Error;
use tonic;
pub mod proto {
    tonic::include_proto!("engine");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051"
    Ok(())
}
