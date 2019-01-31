use rocket::request::{FromRequest, Outcome, Request};

const X_GITHUB_EVENT: &str = "X-GitHub-Event";

const ISSUE_EVENT: &str = "issue";
const ISSUE_COMMENT_EVENT: &str = "issue_comment";
const PULL_REQUEST_EVENT: &str = "pull_request";

#[derive(Clone, Debug, PartialEq)]
pub enum GitHubEvent {
    Issue,
    IssueComment,
    PullRequest,
}

impl<'a, 'r> FromRequest<'a, 'r> for GitHubEvent {
    type Error = failure::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let event = request.headers().get_one(X_GITHUB_EVENT);

        let event = match event {
            Some(e) => e,
            None => {
                return Outcome::Failure((
                    rocket::http::Status::BadRequest,
                    failure::format_err!("{} is not set", X_GITHUB_EVENT),
                ));
            }
        };

        let event = match event {
            ISSUE_EVENT => GitHubEvent::Issue,
            ISSUE_COMMENT_EVENT => GitHubEvent::IssueComment,
            PULL_REQUEST_EVENT => GitHubEvent::PullRequest,
            _ => {
                debug!("{}", event); // TODO delete
                return Outcome::Failure((
                    rocket::http::Status::BadRequest,
                    failure::format_err!("unsuported event"),
                ));
            }
        };

        Outcome::Success(event)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use rocket::http::Header;
    use rocket::local::Client;

    #[test]
    fn test_from_request_issue_comment() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let header = Header::new("X-GitHub-Event", "issue_comment");
        let request = client.post("/").header(header).body("test");

        let event = GitHubEvent::from_request(&request.inner());

        assert!(event.is_success());
        assert_eq!(event.unwrap(), GitHubEvent::IssueComment);
    }

    #[test]
    fn test_from_request_issue() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let header = Header::new("X-GitHub-Event", "issue");
        let request = client.post("/").header(header).body("test");

        let event = GitHubEvent::from_request(&request.inner());

        assert!(event.is_success());
        assert_eq!(event.unwrap(), GitHubEvent::Issue);
    }

    #[test]
    fn test_from_request_pull_request() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let header = Header::new("X-GitHub-Event", "pull_request");
        let request = client.post("/").header(header).body("test");

        let event = GitHubEvent::from_request(&request.inner());

        assert!(event.is_success());
        assert_eq!(event.unwrap(), GitHubEvent::PullRequest);
    }
}
