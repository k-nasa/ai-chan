use crate::github::issue_comment::Issue;
use crate::github::Repository;
use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug)]
pub struct IssueEvent {
    pub action: IssueAction,
    pub issue: Issue,
    pub repository: Repository,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum IssueAction {
    #[serde(rename = "opened")]
    Opened,
    #[serde(rename = "edited")]
    Edited,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "transferred")]
    Transferred,
    #[serde(rename = "pinned")]
    Pinned,
    #[serde(rename = "unpinned")]
    Unpinned,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "reopened")]
    Reopened,
    #[serde(rename = "assigned")]
    Assigned,
    #[serde(rename = "unassigned")]
    Unassigned,
    #[serde(rename = "labeled")]
    Labeled,
    #[serde(rename = "unlabeled")]
    Unlabeled,
    #[serde(rename = "milestoned")]
    Milestoned,
    #[serde(rename = "demilestoned")]
    Demilestoned,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn from_json_str() {
        let event1: IssueEvent = serde_json::from_str(&issue_payload()).unwrap();

        let event2 = IssueEvent {
            action: IssueAction::Edited,
            repository: Repository {
                id: 1,
                name: "Hello-World".to_owned(),
                full_name: "Codertocat/Hello-World".to_owned(),
            },
            issue: Issue {
                id: 1,
                url: "https://api.github.com/repos/Codertocat/Hello-World/issues/2".to_owned(),
                number: 1,

                title: "Spelling error in the README file".to_owned(),

                body: "It looks like you accidently spelled 'commit' with two 't's.".into(),
            },
        };

        assert_eq!(event1, event2);
    }
}
