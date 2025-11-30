use tonic::Request;

pub fn get_uid<T>(req: &Request<T>) -> String {
    req.metadata()
        .get("uid")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

pub fn get_auth<T>(req: &Request<T>) -> String {
    req.metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}
