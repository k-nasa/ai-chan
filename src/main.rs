#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]

mod github;
mod test_support;

#[macro_use]
extern crate log;

type AIChannResult = Result<(), failure::Error>;

use github::{github_event::GitHubEvent, pull_request::PullRequestEvent};
use rocket::{
    config::{Config, Environment, LoggingLevel},
    get, post, routes, Data,
};
use std::io::Read;

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

    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    let config = Config::build(Environment::Development)
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

fn handle_github_webhook(event: GitHubEvent, payload: Data) -> AIChannResult {
    info!("Start hendle {:?} event", event);

    let mut json_string = String::new();
    if payload.open().read_to_string(&mut json_string).is_err() {
        failure::bail!("Bad request. failed read payload.");
    }

    let payload_json: serde_json::Value = serde_json::from_str(&json_string)?;

    match event {
        GitHubEvent::PullRequest => handle_pull_request(payload_json)?,
        GitHubEvent::Issue => warn!("unimplemented!!"),
        GitHubEvent::IssueComment => warn!("unimplemented"),
    }

    Ok(())
}

fn handle_pull_request(json: serde_json::Value) -> AIChannResult {
    unimplemented!()
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
