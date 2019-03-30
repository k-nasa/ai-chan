use crate::command::Command;
use crate::github::issue_comment::{IssueCommentAction, IssueCommentEvent};
use crate::AIChannResult;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let issue_comment_event: IssueCommentEvent = serde_json::from_value(json)?;

    if issue_comment_event.action == IssueCommentAction::Deleted {
        warn!("Unsupport comment {:?} action", issue_comment_event.action);
        return Ok(());
    }

    let command = Command::parse_command(&issue_comment_event.comment.body)?;

    match command {
        Command::UserAssign(_) => command.exec_command_assignee_to_pr(
            issue_comment_event.issue.number,
            issue_comment_event.repository,
        )?,
        Command::ApprovalPR(_) => command.exec_command_approval(issue_comment_event.clone())?,
        Command::RandAssign => Command::exec_command_rand_assignee_to_pr(
            issue_comment_event.issue.number,
            issue_comment_event.repository,
        )?,
        Command::MergeUpstream(_) => (),
    }

    Ok(())
}
