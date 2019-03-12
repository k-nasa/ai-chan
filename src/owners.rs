use crate::config::Config;
use hubcaps::{Credentials, Github};
use rand::Rng;
use serde_derive::*;
use tokio::runtime::Runtime;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Owners {
    pub reviewers: Vec<String>,
    pub delete_branch: Option<bool>,
    rand_assigne: Option<bool>,
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
            failure::bail!("failed import owners.toml:{}", e);
        }

        let content: Vec<u8> = file.unwrap().content.into();
        let content = String::from_utf8(content)?;

        let owners = toml::from_str(&content)?;

        Ok(owners)
    }

    pub fn is_delete_branch_some_true(&self) -> bool {
        match self.delete_branch {
            Some(true) => true,
            _ => false,
        }
    }

    pub fn rand_assigne(&self) -> bool {
        self.rand_assigne.unwrap_or(false)
    }

    pub fn pick_assignee(&self) -> Option<&String> {
        let index: usize = rand::thread_rng().gen_range(0, self.reviewers.len());

        self.reviewers.get(index)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_toml_string() {
        let toml = r###"
            reviewers = ["k-nasa"]
            rand_assigne = true
            "###;

        let owners = Owners {
            reviewers: vec!["k-nasa".to_string()],
            delete_branch: None,
            rand_assigne: Some(true),
        };

        assert_eq!(owners, toml::from_str(toml).unwrap());

        let toml = r###"
            reviewers = ["k-nasa"]
            delete_branch = true
            "###;

        let owners = Owners {
            reviewers: vec!["k-nasa".to_string()],
            rand_assigne: None,
            delete_branch: Some(true),
        };

        assert_eq!(owners, toml::from_str(toml).unwrap());
    }

    #[test]
    fn pick_assignee() {
        let owners = Owners {
            reviewers: vec!["k-nasa".into()],
            rand_assigne: None,
            delete_branch: None,
        };

        assert_eq!(owners.pick_assignee(), Some(&"k-nasa".to_string()))
    }

    #[test]
    fn pick_assignee_from_many() {
        let owners = Owners {
            reviewers: vec!["k-nasa".into(), "ai-chan".into()],
            delete_branch: None,
            rand_assigne: None,
        };

        let mut rand_k_nasa = false;
        let mut rand_ai_chan = false;

        for _ in 0..10 {
            let picked = owners.pick_assignee();
            if picked == Some(&String::from("k-nasa")) {
                rand_k_nasa = true
            }

            if picked == Some(&String::from("ai-chan")) {
                rand_ai_chan = true
            }

            assert!(rand_k_nasa || rand_ai_chan);
        }
    }
}
