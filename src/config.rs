use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigTomlServer {
    pub cgrpc_token: Option<String>, // Administrator Token, used to invoke cgrpc reqs. If not preset will default to no protection.
    pub port: Option<String>,
}
impl Default for ConfigTomlServer {
    fn default() -> Self {
        Self {
            port: Some("[::1]:50051".into()),
            cgrpc_token: None,
        }
    }
}
