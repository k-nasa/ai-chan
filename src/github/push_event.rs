use super::Repository;
use serde_derive::*;

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub ref_string: String,
    pub repository: Repository,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn from_json_str() {
        let event1: PushEvent = serde_json::from_str(&push_event_payload()).unwrap();

        let event2 = PushEvent {
            ref_string: "refs/tags/simple-tag".to_string(),
            repository: Repository {
                id: 135_493_233,
                name: "Hello-World".to_string(),
                full_name: "Codertocat/Hello-World".to_string(),
            },
        };

        assert_eq!(event1, event2);
    }
}
