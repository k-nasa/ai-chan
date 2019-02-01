#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]

mod config;
mod github;
mod request_handle;
mod test_support;

#[macro_use]
extern crate log;

type AIChannResult = Result<(), failure::Error>;

use github::github_event::GitHubEvent;
use request_handle::handle_github_webhook;
use rocket::{
    config::{Environment, LoggingLevel},
    get, post, routes, Data,
};

fn main() {
    std::env::set_var("RUST_LOG", "ai_chan");
    env_logger::init();

    info!("===== ai-chann ===================================");
    info!("start server");
    info!("address: {}", "localhost");
    info!("listen http on port: {}", 8000);
    info!("botname for GitHub: {}", "ai-chann");
    info!("Server has launched from http://{}:{}", "localhost", 8000);
    info!("===================================================");

    rocket(config).launch();
}

fn rocket(config: crate::config::Config) -> rocket::Rocket {
    let config = rocket::config::Config::build(Environment::Development)
        .address(config.address())
        .port(*config.port() as u16)
        .log_level(LoggingLevel::Off)
        .finalize()
        .unwrap();

    rocket::custom(config).mount("/", routes![index, github])
}

#[get("/")]
fn index() -> &'static str {
    // FIXME どうせなら使い方を出したほうが良いのでは？
    "Hello, world!"
}

#[post("/github", format = "application/json", data = "<payload>")]
fn github(event: Result<GitHubEvent, failure::Error>, payload: Data) {
    if let Err(e) = event {
        warn!("{}", e);
        return;
    }

    let result = handle_github_webhook(event.unwrap(), payload);

    match result {
        Ok(_) => info!("Sucess request handle"),
        Err(e) => error!("Failed request handle: {}", e),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn test_index() {
        let client = Client::new(rocket(Config::default())).expect("valid rocket instance");

        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
    }
}
