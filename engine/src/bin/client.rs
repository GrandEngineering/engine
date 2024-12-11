use enginelib::api::EngineAPI;
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

    let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)>;
    let lib = unsafe {
        let library = Library::new("target/debug/libengine_core.so").unwrap();
        let run: Symbol<unsafe extern "Rust" fn(reg: &mut EngineAPI)> =
            library.get(b"run").unwrap();
        run(&mut api);
        library // Return the library to keep it in scope
    };
    std::mem::forget(lib);

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
