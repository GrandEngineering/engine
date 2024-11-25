//use enginelib::EventHandler;
use enginelib::{event, Registry, Task};
use libloading::Library;
use libloading::Symbol;
use prost::Message;
use std::error::Error;
use std::{collections::HashMap, fmt::Debug, ops::DerefMut, sync::Arc};

pub mod proto {
    tonic::include_proto!("engine");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";

    let mut api = event::EngineAPI::default();
    unsafe {
        let lib = Library::new("target/debug/libengine_core.so").unwrap();
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut event::EngineAPI)> =
            lib.get(b"run").unwrap();
        run(&mut api);
    }

    // let mut client = EngineClient::connect(url).await?;
    let req = proto::TaskRequest {
        task_id: "namespace:fib".to_string().encode_to_vec(),
    };
    let request = tonic::Request::new(req);
    // let res = client.aquire_task(request).await?;
    // let data = res.get_ref();
    let tsk: Arc<dyn Task> = api
        .task_registry
        .get(&("namespace".to_string(), "fib".to_string()))
        .unwrap();

    Ok(())
}
