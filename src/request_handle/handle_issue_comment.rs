use crate::github::issue_comment::IssueCommentEvent;
use crate::AIChannResult;

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
    Ok(())
}
