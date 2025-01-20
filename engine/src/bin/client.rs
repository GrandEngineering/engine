use bincode::deserialize;
use enginelib::api::EngineAPI;
use enginelib::plugin::LibraryManager;
use enginelib::task::Task;
use enginelib::RawIdentier;
use proto::engine_client;
//use enginelib::EventHandler;
use enginelib::Registry;
use libloading::Library;
use libloading::Symbol;
use prost::Message;
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
    let tasks: Vec<RawIdentier> = deserialize(&vec).unwrap();
    tasks.iter().for_each(|task| {
        println!("Task: {:?}", task);
    });
    Ok(())
}
