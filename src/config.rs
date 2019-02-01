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
}

fn home_dir_string() -> Result<String, failure::Error> {
    match dirs::home_dir() {
        Some(dir) => Ok(dir.to_str().unwrap().to_owned()),
        _ => failure::bail!("Failed get home directory"),
    }
}
