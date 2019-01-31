use rocket::request::{FromRequest, Outcome, Request};

#[derive(Clone, Debug, PartialEq)]
pub enum GitHubEvent {
    IssueComment,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use rocket::http::Header;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn test_from_request() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let header = Header::new("X-GitHub-Event", "issue_comment");
        let request = client.post("/").header(header).body("test");

        let event = GitHubEvent::from_request(&request.inner());

        assert!(event.is_success());
        assert_eq!(event.unwrap(), GitHubEvent::IssueComment);
    }
}
