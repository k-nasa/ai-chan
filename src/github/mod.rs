pub mod github_event;
pub mod issue;
pub mod issue_comment;
pub mod pull_request;

use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug)]
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct User {
    pub id: u32,
    pub login: String,
}
