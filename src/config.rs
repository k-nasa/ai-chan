use serde_derive::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    port: u32,
    address: String,
    botname: String,
    github_api_key: String,
}
