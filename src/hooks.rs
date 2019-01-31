use rocket::request::{FromRequest, Outcome, Request};

#[derive(Clone, Debug, PartialEq)]
pub enum GitHubEvent {
    IssueComment,
}
