use crate::config::Config;
use crate::github::api::*;
use crate::github::issue_comment::*;
use crate::github::Repository;
use crate::owners::Owners;
use crate::{AIChanError, AIChannResult};

type BotName = String;
type Assignees = Vec<String>;
type BranchName = String;

#[derive(PartialEq, Debug)]
pub enum Command {
    ApprovalPR(BotName),
    UserAssign(Assignees),
    RandAssign,
    MergeUpstream(BranchName),
}

impl Command {
    pub async fn exec_command_assignee_to_pr(
        self,
        number: u32,
        repository: Repository,
    ) -> AIChannResult {
        let user_assign = self.user_assign();

        if user_assign.is_none() {
            return Err(Box::new(AIChanError("Faild parse command".to_string())));
        }

        let assignees = user_assign.unwrap();
        let label_name = vec!["S-awaiting-review"];

        add_assignees_to_pr(number, &repository, &assignees).await?;
        add_label(number, &repository, label_name).await?;

        info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

        Ok(())
    }

    pub async fn exec_command_rand_assignee_to_pr(
        number: u32,
        repository: Repository,
    ) -> AIChannResult {
        let owners = Owners::from_repository(&repository.full_name).await?;
        let assignees = owners.pick_assignee();
        let label_name = vec!["S-awaiting-review"];

        add_comment(number, &repository, "Assign reviewers randomly").await?;

        let assignees: Vec<String> = if let Some(assignee) = assignees {
            vec![assignee.to_string()]
        } else {
            return Err(Box::new(AIChanError("Unset reviewers".to_string())));
        };

        add_assignees_to_pr(number, &repository, &assignees).await?;
        add_label(number, &repository, label_name).await?;

        info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

        Ok(())
    }

    pub async fn exec_command_assignee_to_issue(
        self,
        number: u32,
        repository: Repository,
    ) -> AIChannResult {
        let user_assign = self.user_assign();

        if user_assign.is_none() {
            return Err(Box::new(AIChanError("Failed parse command".to_string())));
        }

        let assignees = user_assign.unwrap();

        add_assignees_to_issue(number, &repository, &assignees).await?;

        info!("Add assignees {:?} to PullRequest#{}", &assignees, number);

        Ok(())
    }

    pub async fn exec_command_approval(
        self,
        issue_comment_event: IssueCommentEvent,
    ) -> AIChannResult {
        let botname = self.approval_pr();
        if botname.is_none() {
            return Err(Box::new(AIChanError("Failed parse command".to_string())));
        }

        let botname = botname.unwrap();
        let config = Config::load_config().unwrap_or_default();
        if botname != config.botname() {
            return Err(Box::new(AIChanError("Invalid botname".to_string())));
        }

        let repository_full_name = &issue_comment_event.repository.full_name;
        let username = &issue_comment_event.comment.user.login;

        let owners = Owners::from_repository(repository_full_name).await?;
        if !owners.reviewers.iter().any(|r| username == r) {
            return Err(Box::new(AIChanError("No merge permission".to_string())));
        }

        let number = issue_comment_event.issue.number;
        let repository = issue_comment_event.repository.clone();
        let repo = issue_comment_event.repository.full_name.clone();

        merge_repository(issue_comment_event).await?;

        if owners.is_delete_branch_some_true() {
            let result = delete_branch(&repo, number).await;
            debug!("{:?}", result);
            add_comment(number, &repository, "Delete branch automatically").await?;
        }

        Ok(())
    }

    pub async fn exec_command_merge_upstream(
        head_branch: String,
        repository: Repository,
        number: u32,
    ) -> AIChannResult {
        let pr = fetch_pull_request(number, &repository).await?;
        let base_branch = pr.head.ref_string;

        add_comment(
            number,
            &repository,
            &format!("Try auto merge {} into {}", base_branch, head_branch),
        )
        .await?;

        if merge_branch(&base_branch, &head_branch, &repository)
            .await
            .is_err()
        {
            add_comment(
                number,
                &repository,
                &format!(
                    "Sorry. Failed auto merge {} into {} :sob",
                    base_branch, head_branch
                ),
            )
            .await?;
        }

        Ok(())
    }

    // FIXME 可読性が低い
    pub fn parse_command(body: &str) -> Result<Command, crate::Error> {
        let input: Vec<&str> = body
            .lines()
            .filter(|l| {
                l.contains("r?") || l.contains("r+") || l.contains("rand?") || l.contains("merge+")
            })
            .collect();

        if input.is_empty() {
            return Err(Box::new(AIChanError("Not input".to_string())));
        }

        let command_line: Vec<&str> = input[0].split_whitespace().collect();
        let (head, tail) = command_line.split_at(1);

        if Some(&"r?") == head.first() {
            let assignees: Vec<String> = tail
                .iter()
                .filter(|a| a.starts_with('@'))
                .map(|a| a.trim_start_matches('@'))
                .map(std::borrow::ToOwned::to_owned)
                .collect();

            if assignees.is_empty() {
                return Err(Box::new(AIChanError("Not Found username".to_string())));
            }

            return Ok(Command::UserAssign(assignees));
        }

        if Some(&"rand?") == head.first() {
            return Ok(Command::RandAssign);
        }

        if Some(&"merge+") == head.first() {
            let branch_name = tail.first().unwrap_or(&"master");

            return Ok(Command::MergeUpstream(branch_name.to_string()));
        }

        if let Some(botname) = head.first() {
            if !botname.starts_with('@') {
                return Err(Box::new(AIChanError("Not Found valid command".to_string())));
            }

            if tail[0] == "r+" {
                let botname = botname.trim_start_matches('@');
                return Ok(Command::ApprovalPR(botname.to_string()));
            }
        }

        return Err(Box::new(AIChanError("Not Found valid command".to_string())));
    }

    pub fn is_user_assign(&self) -> bool {
        match self {
            Command::UserAssign(_) => true,
            _ => false,
        }
    }

    pub fn is_rand_assign(&self) -> bool {
        match self {
            Command::RandAssign => true,
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

    #[test]
    fn should_parse_rand_keyword() {
        let body = "rand?";
        let command = Command::RandAssign;
        assert_eq!(Command::parse_command(&body).unwrap(), command);
    }

    #[test]
    fn should_parse_maege_upstream() {
        let body = "merge+";
        let command = Command::MergeUpstream("master".to_string());

        assert_eq!(Command::parse_command(&body).unwrap(), command);
    }

    #[test]
    fn should_parse_maege_upstream_when_include_branch() {
        let body = "merge+ branch";
        let command = Command::MergeUpstream("branch".to_string());

        assert_eq!(Command::parse_command(&body).unwrap(), command);
    }
}
