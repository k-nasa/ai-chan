#[derive(Clone, Debug, PartialEq)]
pub enum GitHubEvent {
    Push,
    Issue,
    IssueComment,
    PullRequest,
    Iregal(String),
}

impl From<String> for GitHubEvent {
    fn from(s: String) -> Self {
        const PUSH_EVENT: &str = "push";
        const ISSUE_EVENT: &str = "issues";
        const ISSUE_COMMENT_EVENT: &str = "issue_comment";
        const PULL_REQUEST_EVENT: &str = "pull_request";

        match s.as_str() {
            ISSUE_EVENT => GitHubEvent::Issue,
            ISSUE_COMMENT_EVENT => GitHubEvent::IssueComment,
            PULL_REQUEST_EVENT => GitHubEvent::PullRequest,
            PUSH_EVENT => GitHubEvent::Push,
            _ => GitHubEvent::Iregal(s),
        }
    }
}
