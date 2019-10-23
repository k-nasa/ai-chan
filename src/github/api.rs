use crate::config::Config;
use crate::github::issue_comment::*;
use crate::github::pull_request::*;
use crate::github::Repository;
use crate::{AIChannResult, Error};

use surf::{http, http::method::Method, url};
use std::collections::HashMap;
use tokio::runtime::Runtime;

fn github_client(
    method: http::Method,
    url: String,
) -> Result<surf::Request<impl surf::middleware::HttpClient>, Error> {
    let url = url::Url::parse(&format!("https://api.github.com{}", url))?;

    // FIXME 毎回ファイル読み込みが走る
    let token = Config::load_config()
        .unwrap_or_default()
        .github_api_key()
        .to_string();

    Ok(surf::Request::new(method, url).set_header("Authorization", format!("token {}", token)))
}

pub(crate) async fn delete_branch(repo: &str, number: u32) -> AIChannResult {
    let repo = repo.split('/').collect::<Vec<&str>>();

    let pull: PullRequest = github_client(
        Method::GET,
        format!("/repos/{}/{}/pulls/{}", repo[0], repo[1], number),
    )?
    .recv_json()
    .await?;

    info!("{}", pull.head.ref_string);

    github_client(
        Method::DELETE,
        format!(
            "repos/{}/{}/git/refs/{}",
            repo[0], repo[1], pull.head.ref_string
        ),
    )?
    .recv_json()
    .await?;

    Ok(())
}

pub(crate) async fn merge_repository(issue_comment_event: IssueCommentEvent) -> AIChannResult {
    let repo = issue_comment_event.repository.repo_tuple();
    let number = issue_comment_event.issue.number;

    github_client(
        Method::PUT,
        format!("/repos/{}/{}/puls/{}/merge", repo.0, repo.1, number),
    )?
    .recv_string()
    .await?;

    Ok(())
}

pub async fn add_assignees_to_issue(
    number: u32,
    repository: &Repository,
    assignees: &[String],
) -> AIChannResult {
    let repo = repository.repo_tuple();
    let assignees: Vec<&str> = assignees.iter().map(std::convert::AsRef::as_ref).collect();
    let mut body = HashMap::new();
    body.insert("assignees", assignees);

    github_client(Method::PATCH, format!("/repos/{}/{}/issues/{}", repo.0, repo.1, number))?.body_json(&body)?.recv_string().await?;

    Ok(())
}

pub(crate) async fn add_assignees_to_pr(
    number: u32,
    repository: &Repository,
    assignees: &[String],
) -> AIChannResult {
    // FIXME 一旦add_assignees_to_issueをコピペ。違いはないはずなので、どっちかを消して良さそう
    let repo = repository.repo_tuple();
    let assignees: Vec<&str> = assignees.iter().map(std::convert::AsRef::as_ref).collect();
    let mut body = HashMap::new();
    body.insert("assignees", assignees);

    github_client(Method::PATCH, format!("/repos/{}/{}/issues/{}", repo.0, repo.1, number))?.body_json(&body)?.recv_string().await?;

    Ok(())
}

pub(crate) async fn add_label(number: u32, repository: &Repository, labels: Vec<&str>) -> AIChannResult {
    let repo = repository.repo_tuple();
    let mut body = HashMap::new();
    body.insert("labels", labels);

    github_client(Method::PATCH, format!("/repos/{}/{}/issues/{}", repo.0, repo.1, number))?.body_json(&body)?.recv_string().await?;

    Ok(())
}

pub(crate) async fn add_comment(number: u32, repository: &Repository, comment: &str) -> AIChannResult {
    let repo = repository.repo_tuple();
    let mut body = HashMap::new();
    body.insert("body", comment);

    github_client(Method::POST, format!("/repos/{}/{}/issues/{}/comments", repo.0, repo.1, number))?.body_json(&body)?.recv_string().await?;

    Ok(())
}

pub(crate) async fn fetch_pull_request(
    number: u32,
    repository: &Repository,
) -> Result<PullRequest, Error> {
    let repo = repository.repo_tuple();

    let pull: PullRequest = github_client(
        Method::GET,
        format!("/repos/{}/{}/pulls/{}", repo.0, repo.1, number),
    )?
    .recv_json()
    .await?;

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
