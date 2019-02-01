use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequestEvent {
    action: PullRequestAction,
    number: u32,
    pull_request: PullRequest,
    repository: Repository,
}

#[derive(Deserialize, PartialEq, Debug)]
enum PullRequestAction {
    #[serde(rename = "opened")]
    Opened,
    #[serde(rename = "edited")]
    Edited,
    #[serde(rename = "reopened")]
    Reopened,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "assigned")]
    Assigned,
    #[serde(rename = "unassigned")]
    Unassigned,
    #[serde(rename = "review_requested")]
    ReviewRequested,
    #[serde(rename = "review_request_removed")]
    ReviewRequestRemoved,
    #[serde(rename = "labeled")]
    Labeled,
    #[serde(rename = "unlabeled")]
    Unlabeled,
    #[serde(rename = "synchronize")]
    Synchronize,
}

#[derive(Deserialize, PartialEq, Debug)]
struct PullRequest {
    id: u32,
    url: String,
    number: u32,
    state: PullRequestState,
    locked: bool,
    title: String,
    body: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Repository {
    id: u32,
    name: String,
    full_name: String,
}

#[derive(Deserialize, PartialEq, Debug)]
enum PullRequestState {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn from_json_str() {
        let event1: PullRequestEvent =
            serde_json::from_str(&pull_request_webhook_payload()).unwrap();

        let event2 = PullRequestEvent {
            action: PullRequestAction::Closed,
            number: 1,
            repository: Repository {
                id: 135493233,
                name: "Hello-World".to_owned(),
                full_name: "Codertocat/Hello-World".to_owned(),
            },
            pull_request: PullRequest {
                id: 1,
                url: "https://api.github.com/repos/Codertocat/Hello-World/pulls/1".to_owned(),
                number: 1,
                state: PullRequestState::Closed,
                locked: false,
                title: "Update the README with new information".to_owned(),
                body: "This is a pretty simple change that we need to pull into master.".to_owned(),
            },
        };

        assert_eq!(event1, event2);
    }
}
