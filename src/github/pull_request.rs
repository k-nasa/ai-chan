use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequestEvent {
    action: PullRequestAction,
    number: u32,
    pull_request: PullRequest,
}

#[derive(Deserialize, PartialEq, Debug)]
enum PullRequestAction {}

#[derive(Deserialize, PartialEq, Debug)]
pub struct PullRequest {}

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
