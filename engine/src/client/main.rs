use enginelib::*;
use libloading::Library;
use libloading::Symbol;
use prost::Message;
use proto::engine_client::EngineClient;
use std::collections::HashMap;
use std::error::Error;
pub mod proto {
    tonic::include_proto!("engine");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";

    let mut registry = EngineTaskRegistry {
        tasks: HashMap::new(),
    };
    unsafe {
        let lib = Library::new("libengine_core.so").unwrap();
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineTaskRegistry)> =
            lib.get(b"run").unwrap();
        run(&mut registry);
    }

    let mut client = EngineClient::connect(url).await?;
    let req = proto::TaskRequest {
        task_id: "namespace:fib".to_string().encode_to_vec(),
    };
    let request = tonic::Request::new(req);
    let res = client.aquire_task(request).await?;
    let data = res.get_ref();
    Ok(())
}
