mod handle_issue;
mod handle_issue_comment;
mod handle_pull_request;
mod handle_push;

use crate::github::github_event::GitHubEvent;
use crate::AIChannResult;

pub async fn handle_github_webhook(event: GitHubEvent, json_string: &str) -> AIChannResult {
    info!("Start hendle {:?} event", event);

    let payload_json: serde_json::Value = serde_json::from_str(json_string)?;

    match event {
        GitHubEvent::PullRequest => handle_pull_request::exec(payload_json).await?,
        GitHubEvent::IssueComment => handle_issue_comment::exec(payload_json).await?,
        GitHubEvent::Issue => handle_issue::exec(payload_json).await?,
        GitHubEvent::Push => handle_push::exec(payload_json).await?,
    }

    Ok(())
}
