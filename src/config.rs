use serde_derive::*;
use std::fs::*;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    port: u32,
    address: String,
    botname: String,
    github_api_key: String,
}

impl Config {
    pub fn load_config() -> Result<Self, failure::Error> {
        let file_path = Path::new(&home_dir_string()?).join(".config/ai-chan/config.toml");
        let mut file = match File::open(file_path) {
            Ok(c) => c,
            Err(_) => failure::bail!("Please create a file '~/.config/ai-chan/config.toml'"),
        };

        let mut toml_string = String::new();
        file.read_to_string(&mut toml_string)?;

        let config: Self = toml::from_str(&toml_string)?;

        Ok(config)
    }

    pub fn port(&self) -> &u32 {
        &self.port
    }

    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn botname(&self) -> &str {
        &self.botname
    }
    pub fn github_api_key(&self) -> &str {
        &self.github_api_key
    }
}

fn home_dir_string() -> Result<String, failure::Error> {
    match dirs::home_dir() {
        Some(dir) => Ok(dir.to_str().unwrap().to_owned()),
        _ => failure::bail!("Failed get home directory"),
    }
}

impl Default for Config {
    fn default() -> Self {
        let port = std::env::var("PORT")
            .unwrap_or("80".to_owned())
            .parse()
            .unwrap();

        Config {
            port,
            address: "0.0.0.0".to_owned(),
            botname: "ai-chan".to_owned(),
            github_api_key: std::env::var("GITHUB_API_KEY").unwrap_or_default(),
        }
    }
}
