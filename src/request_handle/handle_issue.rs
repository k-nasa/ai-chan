use crate::command::Command;
use crate::github::issue::*;
use crate::AIChannResult;
use serde_json::Value;

pub async fn exec(json: Value) -> AIChannResult {
    let issue_event: IssueEvent = serde_json::from_value(json)?;

    if issue_event.action != IssueAction::Opened {
        warn!("{:?} issue action is unsupport", issue_event.action);

        return Ok(());
    }

    let command = Command::parse_command(&issue_event.issue.body)?;

    if command.is_user_assign() {
        command.exec_command_assignee_to_issue(issue_event.issue.number, issue_event.repository)?;
    }

    Ok(())
}
