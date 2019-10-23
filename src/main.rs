#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]
#![feature(async_closure)]

mod command;
mod config;
mod github;
mod owners;
mod request_handle;
mod test_support;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type AIChannResult = Result<(), Error>;

#[derive(Debug, Clone)]
struct AIChanError(String);

impl std::fmt::Display for AIChanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(error, {})", self.0)
    }
}

impl std::error::Error for AIChanError {
    fn description(&self) -> &str {
        &self.0
    }
}

use crate::config::Config;
use github::github_event::GitHubEvent;
use request_handle::handle_github_webhook;

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

    let mut app = tide::App::new();
    app.at("/ping").get(|_| async move { "pong!" });
    app.at("/github").post(github);

    app.run("127.0.0.1:8000");
}

async fn github(mut cx: tide::Context<()>) {
    const X_GITHUB_EVENT: &str = "X-GitHub-Event";
    const X_HUB_SIGNATURE: &str = "X-Hub-Signature";

    // TODO refactor
    let signature = cx.headers().get(X_HUB_SIGNATURE).unwrap().to_str().unwrap();
    let event_string = String::from(cx.headers().get(X_GITHUB_EVENT).unwrap().to_str().unwrap());
    let event = GitHubEvent::from(event_string);

    let json_string = match cx.body_string().await {
        Err(_) => {
            error!("Bad request. failed read payload.");
            return;
        }
        Ok(s) => s,
    };

    let config = Config::load_config().unwrap_or_default();

    if !config.is_secret_valid(&signature, &json_string) {
        error!("Invalid signe.");
        return;
    }

    async_std::task::block_on(async {
        let result = handle_github_webhook(event, json_string).await;
        match result {
            Ok(_) => info!("Sucess request handle"),
            Err(e) => error!("Failed request handle: {}", e),
        }
    })
}
