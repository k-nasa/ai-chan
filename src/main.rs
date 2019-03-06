#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]

mod command;
mod config;
mod github;
mod owners;
mod request_handle;
mod test_support;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

type AIChannResult = Result<(), failure::Error>;

use crate::config::Config;
use failure::Error;
use github::github_event::{GitHubEvent, Signe};
use request_handle::handle_github_webhook;
use rocket::{
    config::{Environment, LoggingLevel},
    get, post, routes, Data,
};
use std::io::Read;

fn main() {
    std::env::set_var("RUST_LOG", "ai_chan");
    pretty_env_logger::init();

    let config = match crate::config::Config::load_config() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed load config file: {}", e);
            warn!("Using default config");
            crate::config::Config::default()
        }
    };

    // FIXME configに項目を足すたびにここが変わる可能性があるのでなんとかしたい
    info!("===== ai-chann ===================================");
    info!("start server");
    info!("address: {}", config.address());
    info!("listen http on port: {}", config.port());
    info!("botname for GitHub: {}", config.botname());
    info!(
        "Server has launched from http://{}:{}",
        config.address(),
        config.port()
    );
    info!("===================================================");

    rocket(config).launch();
}

fn rocket(config: crate::config::Config) -> rocket::Rocket {
    let config = rocket::config::Config::build(Environment::Production)
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
fn github(signe: Result<Signe, Error>, event: Result<GitHubEvent, Error>, payload: Data) {
    if let Err(e) = event {
        warn!("{}", e);
        return;
    }

    if let Err(e) = signe {
        error!("{}", e);
        return;
    }

    let mut json_string = String::new();
    if payload.open().read_to_string(&mut json_string).is_err() {
        error!("Bad request. failed read payload.");
        return;
    };

    let config = Config::load_config().unwrap_or_default();
    let signature = signe.unwrap().0;

    if !config.is_secret_valid(&signature, &json_string) {
        error!("Invalid signe.");
        return;
    }

    let result = handle_github_webhook(event.unwrap(), &json_string);

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
