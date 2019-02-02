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
pub struct Issue {
    pub id: u32,
    pub url: String,
    pub number: u32,
    pub title: String,
    pub body: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Comment {}
#[cfg(test)]
mod test {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn from_json_str() {
        let event1: IssueCommentEvent = serde_json::from_str(&issue_comment_payload()).unwrap();

        let event2 = IssueCommentEvent {
            action: IssueCommentAction::Created,
            issue: Issue {
                id: 327_883_527,
                url: "https://api.github.com/repos/Codertocat/Hello-World/issues/2".to_owned(),
                number: 2,
                title: "Spelling error in the README file".to_owned(),

                body: "It looks like you accidently spelled 'commit' with two 't's.".to_owned(),
            },
            comment: Comment {
                id: 393_304_133,

                issue_url: "https://api.github.com/repos/Codertocat/Hello-World/issues/2"
                    .to_owned(),

                body: "You are totally right! I'll get this fixed right away.".to_owned(),
            },
            repository: Repository {
                id: 135_493_233,
                name: "Hello-World".to_owned(),
                full_name: "Codertocat/Hello-World".to_owned(),
            },
        };

        assert_eq!(event1, event2);
    }
}
