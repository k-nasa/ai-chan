mod handle_issue_comment;
mod handle_pull_request;

use crate::github::github_event::GitHubEvent;
use crate::AIChannResult;
use rocket::Data;
use std::io::Read;

pub fn handle_github_webhook(event: GitHubEvent, payload: Data) -> AIChannResult {
    info!("Start hendle {:?} event", event);

    let mut json_string = String::new();
    if payload.open().read_to_string(&mut json_string).is_err() {
        failure::bail!("Bad request. failed read payload.");
    }

    let payload_json: serde_json::Value = serde_json::from_str(&json_string)?;

    match event {
        GitHubEvent::PullRequest => handle_pull_request::exec(payload_json)?,
        GitHubEvent::IssueComment => handle_issue_comment::exec(payload_json)?,
        GitHubEvent::Issue => warn!("unimplemented!!"),
    }

    Ok(())
}

enum Commands {
    ApprovalPR,
    UserAssign(UserAssign),
}

struct UserAssign {
    assignees: Vec<String>,
}

// FIXME 可読性が低い
fn parse_command(body: &str) -> Result<Commands, failure::Error> {
    let input: Vec<&str> = body
        .lines()
        // FIXME unimplemented r+
        .filter(|l| l.contains("r?") || l.contains("r+"))
        .collect();

    if input.is_empty() {
        failure::bail!("Not input")
    }

    let command_line: Vec<&str> = input[0].split_whitespace().collect();
    let (head, tail) = command_line.split_at(1);

    if Some(&"r?") == head.first() {
        // TODO rifactor
        let assignees: Vec<String> = tail
            .iter()
            .filter(|a| a.starts_with('@'))
            .map(|a| a.trim_start_matches('@'))
            .map(|a| a.to_owned())
            .collect();

        return Ok(Commands::UserAssign(UserAssign { assignees }));
    }

    failure::bail!("Not Found valid command")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_command() {
        let body = "r? @k-nasa";

        assert_eq!(parse_command(&body), vec!["k-nasa"]);
    }

    #[test]
    fn test_parse_command_when_many_assignees() {
        let body = "r? @k-nasa @k-nasa2";

        assert_eq!(parse_command(&body), vec!["k-nasa", "k-nasa2"]);
    }

    #[test]
    fn test_parse_command_when_many_input() {
        let body = r###"
            This pr is hogehogheo.
            r? @k-nasa

            I think hogehogheo.

            r? @k-nasa2
            "###;

        assert_eq!(parse_command(&body), vec!["k-nasa"]);
    }

    #[test]
    fn test_parse_command_when_invalid() {
        let body1 = "r? ";
        let body2 = "@hoge r?";
        let body3 = "";
        let body4 = "hogehoge";

        assert_eq!(parse_command(&body1), Vec::<&str>::new());
        assert_eq!(parse_command(&body2), Vec::<&str>::new());
        assert_eq!(parse_command(&body3), Vec::<&str>::new());
        assert_eq!(parse_command(&body4), Vec::<&str>::new());
    }
}
