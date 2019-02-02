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

type BotName = String;
type Assignees = Vec<String>;

#[derive(PartialEq, Debug)]
enum Commands {
    ApprovalPR(BotName),
    UserAssign(Assignees),
}

impl Commands {
    pub fn user_assign(self) -> Option<Assignees> {
        match self {
            Commands::UserAssign(u) => Some(u),
            _ => None,
        }
    }

    pub fn approval_pr(self) -> Option<BotName> {
        match self {
            Commands::ApprovalPR(b) => Some(b),
            _ => None,
        }
    }
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

        if assignees.is_empty() {
            failure::bail!("Not Found username")
        }

        return Ok(Commands::UserAssign(assignees));
    }

    if let Some(botname) = head.first() {
        if !botname.starts_with('@') {
            failure::bail!("Not Found valid command")
        }

        if tail[0] == "r+" {
            let botname = botname.trim_start_matches('@');
            return Ok(Commands::ApprovalPR(botname.to_string()));
        }
    }

    failure::bail!("Not Found valid command")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_command_pr_event() {
        let body = "r? @k-nasa";
        let commands = Commands::UserAssign(vec!["k-nasa".to_string()]);

        assert_eq!(parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_many_assignees() {
        let body = "r? @k-nasa @k-nasa2";
        let commands = Commands::UserAssign(vec!["k-nasa".to_string(), "k-nasa2".to_owned()]);

        assert_eq!(parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_many_input() {
        let body = r###"
            This pr is hogehogheo.
            r? @k-nasa

            I think hogehogheo.

            r? @k-nasa2
            "###;

        let commands = Commands::UserAssign(vec!["k-nasa".to_string()]);

        assert_eq!(parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_invalid_pr_event() {
        let body1 = "r? ";
        let body2 = "@hoge r?";
        let body3 = "";
        let body4 = "hogehoge";

        assert!(parse_command(&body1).is_err());
        assert!(parse_command(&body2).is_err());
        assert!(parse_command(&body3).is_err());
        assert!(parse_command(&body4).is_err());
    }

    #[test]
    fn test_parse_command_comment_event() {
        let body = "@botname r+";
        let commands = Commands::ApprovalPR("botname".to_owned());

        assert_eq!(parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_comment_event_when_many_input() {
        let body = r###"
            This pr is hogehogheo.

            I think hogehogheo.

            @botname r+
            "###;

        let commands = Commands::ApprovalPR("botname".to_owned());

        assert_eq!(parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_invalid_comment() {
        let body1 = "r+";
        let body2 = "r+ @hogehgeo";
        let body3 = "";
        let body4 = "hogehoge";

        assert!(parse_command(&body1).is_err());
        assert!(parse_command(&body2).is_err());
        assert!(parse_command(&body3).is_err());
        assert!(parse_command(&body4).is_err());
    }
}
