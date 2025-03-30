use tonic::Request;

#[macro_use]
pub mod macros;

pub fn get_uid<T>(req: &Request<T>) -> String {
    let out = req.metadata().get("uid");
    if let Some(out) = out {
        out.to_str().unwrap().to_string()
    } else {
        "".to_string()
    }
}

pub fn get_auth<T>(req: &Request<T>) -> String {
    let out = req.metadata().get("authorization");
    if let Some(out) = out {
        out.to_str().unwrap().to_string()
    } else {
        "".to_string()
    }
}
