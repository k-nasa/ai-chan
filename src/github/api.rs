use crate::config::Config;
use crate::github::issue_comment::*;
use crate::github::pull_request::*;
use crate::github::Repository;
use crate::owners::Owners;
use crate::AIChannResult;
use hubcaps::{Credentials, Github};
use tokio::runtime::Runtime;

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

pub fn delete_branch(owners: Owners, repo: &str, number: u32) -> AIChannResult {
    if owners.is_some_true() {
        info!("delete_branch setting is nothing");
        return Ok(());
    }

    let repo = repo.split('/').collect::<Vec<&str>>();
    let github = github_client_setup!();

    let mut rt = Runtime::new()?;
    let pull: PullRequest = rt
        .block_on(github.get(&format!("/repos/{}/{}/pulls/{}", repo[0], repo[1], number)))
        .unwrap();

    info!("{}", pull.head.ref_string);

    let result = rt.block_on(
        github
            .repo(repo[0], repo[1])
            .git()
            .delete_reference(format!("heads/{}", pull.head.ref_string)),
    );

    if result.is_err() {
        failure::bail!("Failed delete branch");
    }

    Ok(())
}

pub fn merge_repository(issue_comment_event: IssueCommentEvent) -> AIChannResult {
    let repo = issue_comment_event.repository.repo_tuple();
    let github = github_client_setup!();

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

pub fn add_assignees_to_issue(
    number: u32,
    repository: &Repository,
    assignees: &[String],
) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();
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

pub fn add_assignees_to_pr(
    number: u32,
    repository: &Repository,
    assignees: &[String],
) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();
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
