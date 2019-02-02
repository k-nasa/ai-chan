use super::parse_command;
use crate::config::Config;
use crate::github::issue_comment::{IssueCommentAction, IssueCommentEvent};
use crate::AIChannResult;
use hubcaps::*;
use tokio::runtime::Runtime;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let issue_comment_event: IssueCommentEvent = serde_json::from_value(json)?;

    if issue_comment_event.action == IssueCommentAction::Deleted {
        warn!("Unsupport comment {:?} action", issue_comment_event.action);
        return Ok(());
    }

    let command = parse_command(&issue_comment_event.comment.body)?;
    let botname = command.approval_pr();
    if botname.is_none() {
        failure::bail!("Faild parse command");
    }

    let botname = botname.unwrap();
    let config = Config::load_config()?;
    if botname != config.botname() {
        failure::bail!("Invalid botname");
    }

    merge_repository(issue_comment_event)?;

    Ok(())
}

fn merge_repository(issue_comment_event: IssueCommentEvent) -> AIChannResult {
    let repo = issue_comment_event
        .repository
        .full_name
        .split('/')
        .collect::<Vec<&str>>();

    let config = Config::load_config()?;

    let github = Github::new(
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
        Credentials::Token(config.github_api_key().to_owned()),
    );

    let mut rt = Runtime::new()?;
    rt.block_on(
        github
            .repo(repo[0], repo[1])
            .pulls()
            .get(issue_comment_event.issue.number.into())
            .merge(),
    )
    .unwrap();

    Ok(())
}
