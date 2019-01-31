#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]

mod hooks;

#[macro_use]
extern crate log;

use rocket::{
    config::{Config, Environment, LoggingLevel},
    get, post, routes, Data,
};
use std::io::Read;

#[get("/")]
fn index() -> &'static str {
    // FIXME どうせなら使い方を出したほうが良いのでは？
    "Hello, world!"
}

fn main() {
    std::env::set_var("RUST_LOG", "ai_chan");
    env_logger::init();

    info!("===== ai-chann =====");
    info!("start server");
    info!("address: {}", "localhost");
    info!("listen http on port: {}", 8000);
    info!("botname for GitHub: {}", "ai-chann");
    info!("Server has launched from http://{}:{}", "localhost", 8000);
    info!("====================");

    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .log_level(LoggingLevel::Off)
        .finalize()
        .unwrap();

    rocket::custom(config).mount("/", routes![index, github])
}

#[post("/github", format = "application/json", data = "<payload>")]
fn github(event: Option<hooks::GitHubEvent>, payload: Data) {
    debug!("{:?}", event); // TODO delete

    if event.is_none() {
        warn!("unsuported event");
        return;
    }

    let mut string = String::new();
    if payload.open().read_to_string(&mut string).is_err() {
        error!("load error");
    }

    let json: serde_json::Value = serde_json::from_str(&string).unwrap_or_default();
    debug!("{:?}", json);
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn test_index() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
    }
}
