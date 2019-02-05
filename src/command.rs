use crate::config::Config;
use crate::github::issue_comment::*;
use crate::github::pull_request::*;
use crate::github::Repository;
use crate::owners::Owners;
use crate::AIChannResult;
use hubcaps::{Credentials, Github};
use tokio::runtime::Runtime;

type BotName = String;
type Assignees = Vec<String>;

#[derive(PartialEq, Debug)]
pub enum Command {
    ApprovalPR(BotName),
    UserAssign(Assignees),
}

macro_rules! github_client_setup {
    () => {
        Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(
                Config::load_config()
                    .unwrap_or_default()
                    .github_api_key()
                    .to_owned(),
            ),
        )
    };
}

impl Command {
    pub fn exec_command_assignee_to_pr(self, number: u32, repository: Repository) -> AIChannResult {
        let user_assign = self.user_assign();

        if user_assign.is_none() {
            failure::bail!("Faild parse command");
        }

        let assignees = user_assign.unwrap();

        Self::add_assignees_to_pr(number, &repository, &assignees)?;

        info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

        Ok(())
    }

    pub fn exec_command_assignee_to_issue(
        self,
        number: u32,
        repository: Repository,
    ) -> AIChannResult {
        let user_assign = self.user_assign();

        if user_assign.is_none() {
            failure::bail!("Faild parse command");
        }

        let assignees = user_assign.unwrap();

        Self::add_assignees_to_issue(number, &repository, &assignees)?;

        info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

        Ok(())
    }

    pub fn exec_command_approval(self, issue_comment_event: IssueCommentEvent) -> AIChannResult {
        let botname = self.approval_pr();
        if botname.is_none() {
            failure::bail!("Faild parse command");
        }

        let botname = botname.unwrap();
        let config = Config::load_config().unwrap_or_default();
        if botname != config.botname() {
            failure::bail!("Invalid botname");
        }

        let repository_full_name = &issue_comment_event.repository.full_name;
        let username = &issue_comment_event.comment.user.login;

        let owners = Owners::from_repository(repository_full_name)?;
        if !owners.reviewers.iter().any(|r| username == r) {
            failure::bail!("No merge permission");
        }

        let number = issue_comment_event.issue.number;
        let repo = issue_comment_event.repository.full_name.clone();

        Self::merge_repository(issue_comment_event)?;
        Self::delete_branch(owners, &repo, number)?;

        Ok(())
    }

    fn delete_branch(owners: Owners, repo: &str, number: u32) -> AIChannResult {
        if owners.is_some_true() {
            info!("delete_branch setting is nothing");
            return Ok(());
        }

        let repo = repo.split('/').collect::<Vec<&str>>();
        let config = Config::load_config().unwrap_or_default();
        let github = Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(config.github_api_key().to_owned()),
        );

        let mut rt = Runtime::new()?;
        let pull: PullRequest = rt
            .block_on(github.get(&format!("/repos/{}/{}/pulls/{}", repo[0], repo[1], number)))
            .unwrap();

        info!("{}", pull.head.ref_string);

        let is_err = rt
            .block_on(
                github
                    .repo(repo[0], repo[1])
                    .git()
                    .delete_reference(format!("heads/{}", pull.head.ref_string)),
            )
            .is_err();

        if is_err {
            failure::bail!("Failed delete branch");
        }

        Ok(())
    }

    fn merge_repository(issue_comment_event: IssueCommentEvent) -> AIChannResult {
        let repo = issue_comment_event.repository.repo_tuple();

        let config = Config::load_config().unwrap_or_default();

        let github = Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(config.github_api_key().to_owned()),
        );

        let mut rt = Runtime::new()?;
        rt.block_on(
            github
                .repo(repo.0, repo.1)
                .pulls()
                .get(issue_comment_event.issue.number.into())
                .merge(),
        )
        .unwrap();

        Ok(())
    }

    fn add_assignees_to_issue(
        number: u32,
        repository: &Repository,
        assignees: &[String],
    ) -> AIChannResult {
        let repo = repository.repo_tuple();

        let config = Config::load_config().unwrap_or_default();

        let github = Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(config.github_api_key().to_owned()),
        );

        let assignees: Vec<&str> = assignees.iter().map(|s| s.as_ref()).collect();

        let mut rt = Runtime::new()?;
        rt.block_on(
            github
                .repo(repo.0, repo.1)
                .issues()
                .get(number.into())
                .assignees()
                .add(assignees),
        )
        .unwrap(); //FIXME unwrap()

        Ok(())
    }

    fn add_assignees_to_pr(
        number: u32,
        repository: &Repository,
        assignees: &[String],
    ) -> AIChannResult {
        let repo = repository.repo_tuple();
        let config = Config::load_config().unwrap_or_default();
        let github = Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(config.github_api_key().to_owned()),
        );

        let assignees: Vec<&str> = assignees.iter().map(|s| s.as_ref()).collect();

        let mut rt = Runtime::new()?;
        rt.block_on(
            github
                .repo(repo.0, repo.1)
                .pulls()
                .get(number.into())
                .assignees()
                .add(assignees),
        )
        .unwrap(); //FIXME unwrap()

        Ok(())
    }
    // FIXME 可読性が低い
    pub fn parse_command(body: &str) -> Result<Command, failure::Error> {
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

            return Ok(Command::UserAssign(assignees));
        }

        if let Some(botname) = head.first() {
            if !botname.starts_with('@') {
                failure::bail!("Not Found valid command")
            }

            if tail[0] == "r+" {
                let botname = botname.trim_start_matches('@');
                return Ok(Command::ApprovalPR(botname.to_string()));
            }
        }

        failure::bail!("Not Found valid command")
    }

    pub fn is_user_assign(&self) -> bool {
        match self {
            Command::UserAssign(_) => true,
            _ => false,
        }
    }

    pub fn is_approval_pr(&self) -> bool {
        match self {
            Command::ApprovalPR(_) => true,
            _ => false,
        }
    }

    pub fn user_assign(self) -> Option<Assignees> {
        match self {
            Command::UserAssign(u) => Some(u),
            _ => None,
        }
    }

    pub fn approval_pr(self) -> Option<BotName> {
        match self {
            Command::ApprovalPR(b) => Some(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_command_pr_event() {
        let body = "r? @k-nasa";
        let commands = Command::UserAssign(vec!["k-nasa".to_string()]);

        assert_eq!(Command::parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_many_assignees() {
        let body = "r? @k-nasa @k-nasa2";
        let commands = Command::UserAssign(vec!["k-nasa".to_string(), "k-nasa2".to_owned()]);

        assert_eq!(Command::parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_many_input() {
        let body = r###"
            This pr is hogehogheo.
            r? @k-nasa

            I think hogehogheo.

            r? @k-nasa2
            "###;

        let commands = Command::UserAssign(vec!["k-nasa".to_string()]);

        assert_eq!(Command::parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_invalid_pr_event() {
        let body1 = "r? ";
        let body2 = "@hoge r?";
        let body3 = "";
        let body4 = "hogehoge";

        assert!(Command::parse_command(&body1).is_err());
        assert!(Command::parse_command(&body2).is_err());
        assert!(Command::parse_command(&body3).is_err());
        assert!(Command::parse_command(&body4).is_err());
    }

    #[test]
    fn test_parse_command_comment_event() {
        let body = "@botname r+";
        let commands = Command::ApprovalPR("botname".to_owned());

        assert_eq!(Command::parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_comment_event_when_many_input() {
        let body = r###"
            This pr is hogehogheo.

            I think hogehogheo.

            @botname r+
            "###;

        let commands = Command::ApprovalPR("botname".to_owned());

        assert_eq!(Command::parse_command(&body).unwrap(), commands);
    }

    #[test]
    fn test_parse_command_when_invalid_comment() {
        let body1 = "r+";
        let body2 = "r+ @hogehgeo";
        let body3 = "";
        let body4 = "hogehoge";

        assert!(Command::parse_command(&body1).is_err());
        assert!(Command::parse_command(&body2).is_err());
        assert!(Command::parse_command(&body3).is_err());
        assert!(Command::parse_command(&body4).is_err());
    }
}
