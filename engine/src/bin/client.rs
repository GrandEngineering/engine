use enginelib::api::EngineAPI;
use enginelib::plugin::LibraryManager;
use enginelib::task::Task;
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

    let mut api = EngineAPI::default();
    let mut lib_manager = LibraryManager::default();
    lib_manager.register_module("target/debug/libengine_core.so", &mut api);
    //Why rust?
    std::mem::forget(lib_manager);
    // let mut client = EngineClient::connect(url).await?;
    let req = proto::TaskRequest {
        task_id: "namespace:fib".to_string().encode_to_vec(),
    };
    let request = tonic::Request::new(req);
    // let res = client.aquire_task(request).await?;
    // let data = res.get_ref();
    let mut tsk: Box<dyn Task> = api
        .task_registry
        .get(&("namespace".to_string(), "fib".to_string()))
        .unwrap();
    tsk.run_cpu();
    println!("Mystery: {:?}", &tsk);

    Ok(())
}
