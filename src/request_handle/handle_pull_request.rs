use super::parse_command;
use crate::config::Config;
use crate::github::pull_request::PullRequestEvent;
use crate::AIChannResult;
use hubcaps::*;
use tokio::runtime::Runtime;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let pull_request_event: PullRequestEvent = serde_json::from_value(json)?;

    let assignees = parse_command(&pull_request_event.pull_request.body);

    if assignees.is_empty() {
        warn!("Not Found valid command");
        return Ok(());
    }

    add_assignees(&pull_request_event, &assignees)?;

    info!(
        "Add assignees {:?} to PullRequest#{}",
        &assignees, pull_request_event.number
    );

    Ok(())
}

fn add_assignees(pull_request_event: &PullRequestEvent, assignees: &[String]) -> AIChannResult {
    let repo = pull_request_event
        .repository
        .full_name
        .split('/')
        .collect::<Vec<&str>>();

    let config = Config::load_config()?;

    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Credentials::Token(config.github_api_key().to_owned()),
    );

    let assignees: Vec<&str> = assignees.iter().map(|s| s.as_ref()).collect();

    let mut rt = Runtime::new()?;
    rt.block_on(
        github
            .repo(repo[0], repo[1])
            .pulls()
            .get(pull_request_event.number.into())
            .assignees()
            .add(assignees),
    )
    .unwrap(); //FIXME unwrap()

    Ok(())
}
