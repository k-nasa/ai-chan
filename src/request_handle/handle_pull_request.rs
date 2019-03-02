use crate::command::Command;
use crate::config::Config;
use crate::github::api::*;
use crate::github::pull_request::{PullRequestAction, PullRequestEvent};
use crate::github::Repository;
use crate::owners::Owners;
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

    let parse_result = Command::parse_command(&pull_request_event.pull_request.body);

    let config = Config::load_config().unwrap_or_default();
    if config.rand_assigne() && parse_result.is_err() {
        exec_command_rand_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        )?;
        return Ok(());
    }

    let command = parse_result?;
    if command.is_user_assign() {
        command.exec_command_assignee_to_pr(
            pull_request_event.pull_request.number,
            pull_request_event.repository,
        )?;
    }

    Ok(())
}

fn exec_command_rand_assignee_to_pr(number: u32, repository: Repository) -> AIChannResult {
    let owners = Owners::from_repository(&repository.full_name)?;
    let assignees = owners.pick_assignee();
    let label_name = vec!["S-awaiting-review"];

    let assignees: Vec<String> = if let Some(assignee) = assignees {
        vec![assignee.to_string()]
    } else {
        failure::bail!("Unset reviewers")
    };

    add_assignees_to_pr(number, &repository, &assignees)?;
    add_label(number, &repository, label_name)?;

    info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

    Ok(())
}
