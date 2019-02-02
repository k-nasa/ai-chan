use crate::github::Repository;
use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug)]
pub struct IssueCommentEvent {
    pub action: IssueCommentAction,
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum IssueCommentAction {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "edited")]
    Edited,
    #[serde(rename = "deleted")]
    Deleted,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Issue {}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Comment {}
