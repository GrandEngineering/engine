#[macro_export]
macro_rules! get_auth {
    ($request:expr) => {{
        let payload = $request
            .metadata()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        payload
    }};
}
#[macro_export]
macro_rules! get_uid {
    ($request:expr) => {{
        let payload = $request
            .metadata()
            .get("uid")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        payload
    }};
}
