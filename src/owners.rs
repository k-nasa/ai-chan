use crate::config::Config;
use hubcaps::{Credentials, Github};
use serde_derive::*;
use tokio::runtime::Runtime;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Owners {
    pub reviewers: Vec<String>,
}

impl Owners {
    pub fn from_repository(repository_full_name: &str) -> Result<Self, failure::Error> {
        let repo = repository_full_name.split('/').collect::<Vec<&str>>();
        let config = Config::load_config().unwrap_or_default();
        let github = Github::new(
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            Credentials::Token(config.github_api_key().to_owned()),
        );

        let mut rt = Runtime::new()?;
        let file = rt.block_on(github.repo(repo[0], repo[1]).content().file("owners.toml"));

        if let Err(e) = file {
            failure::bail!("{}", e);
        }

        let content: Vec<u8> = file.unwrap().content.into();
        let content = String::from_utf8(content)?;

        let owners = toml::from_str(&content)?;

        Ok(owners)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_toml_string() {
        let toml = r###"reviewers = ["k-nasa"]"###;

        let owners = Owners {
            reviewers: vec!["k-nasa".to_string()],
        };

        assert_eq!(owners, toml::from_str(toml).unwrap());
    }
}
