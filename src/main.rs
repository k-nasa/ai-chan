#![feature(async_await, type_alias_enum_variants, proc_macro_hygiene, decl_macro)]

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

const X_GITHUB_EVENT: &str = "X-GitHub-Event";
const X_HUB_SIGNATURE: &str = "X-Hub-Signature";

fn main() {
    std::env::set_var("RUST_LOG", "ai_chan");
    pretty_env_logger::init();

    let config = match crate::config::Config::load_config() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed load config file: {}", e);
            error!("Using default config");
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

    let mut app = tide::App::new();

    app.at("/").get(async move |_| "running server");
    app.at("/github").post(github);
    app.run(format!("{}:{}", config.address(), config.port())).unwrap();
}

async fn github(mut cx: tide::Context<()>) {

    let ( event, sign ) = {
        let header = cx.headers();
        let event = header.get(X_GITHUB_EVENT);
        let sign = header.get(X_HUB_SIGNATURE);
        if event.is_none() {
            warn!("event header is nothing");
            return;
        }

        if sign.is_none() {
            error!("sign header is nothing");
            return;
        }

        let event_string = event.unwrap().to_str().unwrap().to_string();
        (event_string, sign.unwrap().to_str().unwrap())
    };

    let json_string: String = cx.body_string().await.unwrap();

    // let config = Config::load_config().unwrap_or_default();

    // if !config.is_secret_valid(sign, &json_string) {
    //     error!("Invalid signe.");
    //     return;
    // }

    let result = handle_github_webhook(GitHubEvent::from(event), &json_string);

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
