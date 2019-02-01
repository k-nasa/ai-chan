use crate::github::{github_event::GitHubEvent, pull_request::PullRequestEvent};
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
        GitHubEvent::PullRequest => handle_pull_request(payload_json)?,
        GitHubEvent::Issue => warn!("unimplemented!!"),
        GitHubEvent::IssueComment => warn!("unimplemented"),
    }

    Ok(())
}

fn handle_pull_request(json: serde_json::Value) -> AIChannResult {
    let pull_request: PullRequestEvent = serde_json::from_value(json)?;
    unimplemented!()
}

// FIXME 可読性が低い
fn parse_command(body: &str) -> Vec<&str> {
    let input: Vec<&str> = body
        .lines()
        // FIXME unimplemented r+
        .filter(|l| l.contains("r?") || l.contains("r+"))
        .collect();

    if input.is_empty() {
        return vec![];
    }

    // TODO 最初の行にr?がなくても対応できるようにしたい
    let command_line: Vec<&str> = input[0].split_whitespace().collect();
    let (head, tail) = command_line.split_at(1);

    if Some(&"r?") != head.first() {
        return vec![];
    }

    // TODO rifactor
    tail.iter()
        .filter(|a| a.starts_with('@'))
        .map(|a| a.trim_start_matches('@'))
        .collect()
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
