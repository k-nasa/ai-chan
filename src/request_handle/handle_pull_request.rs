use crate::command::Command;
use crate::github::pull_request::{PullRequestAction, PullRequestEvent};
use crate::owners::Owners;
use crate::AIChannResult;

pub async fn exec(json: serde_json::Value) -> AIChannResult {
    let pull_request_event: PullRequestEvent = serde_json::from_value(json)?;

    if pull_request_event.action != PullRequestAction::Opened {
        warn!(
            "Unsupport pull_request {:?} action",
            pull_request_event.action
        );

        return Ok(());
    }

    let parse_result = Command::parse_command(&pull_request_event.pull_request.body);

    let owners = Owners::from_repository(&pull_request_event.repository.full_name).await?;

    if owners.rand_assigne() && parse_result.is_err() {
        Command::exec_command_rand_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        ).await?;
        return Ok(());
    }

    let command = parse_result?;
    if command.is_user_assign() {
        command.exec_command_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        ).await?;
        return Ok(());
    }

    if command.is_rand_assign() {
        Command::exec_command_rand_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        ).await?;
        return Ok(());
    }

    Ok(())
}
