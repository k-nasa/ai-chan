pub mod api;
pub mod github_event;
pub mod issue;
pub mod issue_comment;
pub mod pull_request;
pub mod push_event;

use serde_derive::*;

// FIXME move module
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Repository {
    // XXX add getter
    pub id: u32,
    pub name: String,
    pub full_name: String,
}

impl Repository {
    pub fn repo_tuple(&self) -> (&str, &str) {
        let repo = self.full_name.split('/').collect::<Vec<&str>>();

        (repo[0], repo[1])
    }
}

// FIXME move module
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub login: String,
}
