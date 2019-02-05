use crate::command::Command;
use crate::github::pull_request::{PullRequestAction, PullRequestEvent};
use crate::AIChannResult;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let pull_request_event: PullRequestEvent = serde_json::from_value(json)?;

    if pull_request_event.action != PullRequestAction::Opened {
        warn!(
            "Unsupport pull_request {:?} action",
            pull_request_event.action
        );

        return Ok(());
    }

    let command = Command::parse_command(&pull_request_event.pull_request.body)?;

    if command.is_user_assign() {
        command.exec_command_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        )?;
    }

    Ok(())
}
