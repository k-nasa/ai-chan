use crate::config::Config;
use crate::github::issue_comment::*;
use crate::github::pull_request::*;
use crate::github::Repository;
use crate::AIChannResult;
use serde::de::DeserializeOwned;
use surf::{http, url};
use tokio::runtime::Runtime;

async fn github_client<T: DeserializeOwned>(
    method: http::Method,
    url: &str,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let url = url::Url::parse(&format!("https://api.github.com{}", url))?;

    // FIXME 毎回ファイル読み込みが走る
    let token = Config::load_config()
        .unwrap_or_default()
        .github_api_key()
        .to_string();

    surf::Request::new(method, url)
        .set_header("Authorization", format!("token {}", token))
        .recv_json()
        .await
}

pub async fn delete_branch(repo: &str, number: u32) -> AIChannResult {
    let repo = repo.split('/').collect::<Vec<&str>>();

    let pull: PullRequest = github_client(
        http::method::POST,
        &format!("/repos/{}/{}/pulls/{}", repo[0], repo[1], number),
    )
    .await?;

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
    let assignees: Vec<&str> = assignees.iter().map(std::convert::AsRef::as_ref).collect();

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
    let assignees: Vec<&str> = assignees.iter().map(std::convert::AsRef::as_ref).collect();

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

pub fn add_reviewers_to_pr(
    number: u32,
    repository: &Repository,
    reviewers: &[String],
) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();
    let reviewers: Vec<&str> = reviewers.iter().map(std::convert::AsRef::as_ref).collect();

    let mut map = std::collections::HashMap::new();
    map.insert("reviewers", reviewers);

    let mut rt = Runtime::new()?;
    let result: Result<serde_json::Value, _> = rt.block_on(github.post(
        &format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            repo.0, repo.1, number
        ),
        serde_json::to_vec(&map)?,
    ));

    if result.is_err() {
        failure::bail!("Failed add reviewers: {:?}", result);
    }

    Ok(())
}

pub fn add_label(number: u32, repository: &Repository, labels: Vec<&str>) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();

    let mut rt = Runtime::new()?;
    let result = rt.block_on(
        github
            .repo(repo.0, repo.1)
            .pulls()
            .get(number.into())
            .labels()
            .add(labels),
    );

    if result.is_err() {
        failure::bail!("Failed add labels");
    }

    Ok(())
}

pub fn add_comment(number: u32, repository: &Repository, comment: &str) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();

    let issue = github.repo(repo.0, repo.1).issues().get(u64::from(number));
    let f = issue.comments().create(&CommentOptions {
        body: comment.to_string(),
    });

    let mut rt = Runtime::new()?;
    let result = rt.block_on(f);

    if result.is_err() {
        failure::bail!("Failed add comment");
    }

    Ok(())
}

pub fn fetch_pull_request(
    number: u32,
    repository: &Repository,
) -> Result<PullRequest, failure::Error> {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();
    let mut rt = Runtime::new()?;
    let pull: PullRequest = rt
        .block_on(github.get(&format!("/repos/{}/{}/pulls/{}", repo.0, repo.1, number)))
        .unwrap();

    Ok(pull)
}

pub fn merge_branch(
    base_branch: &str,
    head_branch: &str,
    repository: &Repository,
) -> AIChannResult {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();

    let mut map = std::collections::HashMap::new();
    map.insert("base", base_branch);
    map.insert("head", head_branch);

    let mut rt = Runtime::new()?;
    let result: Result<serde_json::Value, _> = rt.block_on(github.post(
        &format!("/repos/{}/{}/merges", repo.0, repo.1),
        serde_json::to_vec(&map)?,
    ));

    if result.is_err() {
        failure::bail!("Failed merge branch: {:?}", result);
    }

    Ok(())
}

type PullRequests = Vec<PullRequest>;
pub fn fetch_all_pulls_numbers(repository: &Repository) -> Result<Vec<u32>, failure::Error> {
    let repo = repository.repo_tuple();
    let github = github_client_setup!();

    let mut rt = Runtime::new()?;
    let result: Result<PullRequests, _> =
        rt.block_on(github.get(&format!("/repos/{}/{}/pulls", repo.0, repo.1)));

    if result.is_err() {
        failure::bail!("Failed merge branch: {:?}", result);
    }

    let pulls = result.unwrap();

    let mut numbers = Vec::new();

    for pull in pulls {
        numbers.push(pull.number);
    }

    Ok(numbers)
}
