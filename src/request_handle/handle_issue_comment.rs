use super::parse_command;
use crate::github::issue_comment::{IssueCommentAction, IssueCommentEvent};
use crate::AIChannResult;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let issue_comment_event: IssueCommentEvent = serde_json::from_value(json)?;

    if issue_comment_event.action == IssueCommentAction::Deleted {
        warn!("Unsupport comment {:?} action", issue_comment_event.action);
        return Ok(());
    }

    let command = parse_command(&issue_comment_event.comment.body)?;

    if command.is_user_assign() {
        command.exec_command_assignee(
            issue_comment_event.issue.number,
            issue_comment_event.repository,
        )?;
        return Ok(());
    };

    if command.is_approval_pr() {
        command.exec_command_approval(issue_comment_event)?;
    }

    Ok(())
}
