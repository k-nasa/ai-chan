use crate::command::Command;
use crate::github::issue_comment::{IssueCommentAction, IssueCommentEvent};
use crate::AIChannResult;

pub async fn exec(json: serde_json::Value) -> AIChannResult {
    let issue_comment_event: IssueCommentEvent = serde_json::from_value(json)?;

    if issue_comment_event.action == IssueCommentAction::Deleted {
        warn!("Unsupport comment {:?} action", issue_comment_event.action);
        return Ok(());
    }

    debug!("{:?}", issue_comment_event.comment.body);

    let command = Command::parse_command(&issue_comment_event.comment.body)?;

    match command {
        Command::UserAssign(_) => {
            command
                .exec_command_assignee_to_pr(
                    issue_comment_event.issue.number,
                    issue_comment_event.repository,
                )
                .await?
        }
        Command::ApprovalPR(_) => {
            command
                .exec_command_approval(issue_comment_event.clone())
                .await?
        }
        Command::RandAssign => Command::exec_command_rand_assignee_to_pr(
            issue_comment_event.issue.number,
            issue_comment_event.repository,
        ).await?,
        Command::MergeUpstream(base_branch) => Command::exec_command_merge_upstream(
            base_branch,
            issue_comment_event.repository,
            issue_comment_event.issue.number,
        ).await?,
    }

    Ok(())
}
