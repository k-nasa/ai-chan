use crate::github::issue_comment::IssueCommentEvent;
use crate::AIChannResult;

pub fn exec(json: serde_json::Value) -> AIChannResult {
    let issue_comment_event: IssueCommentEvent = serde_json::from_value(json)?;
    Ok(())
}
