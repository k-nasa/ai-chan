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
