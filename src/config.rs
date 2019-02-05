use crypto::{hmac::Hmac, mac::Mac, sha1::Sha1};
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
    secret: String,
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

    // XXX Be sure to refactor
    // TODO Add test code
    pub fn is_secret_valid(&self, signature: &str, payload: &str) -> bool {
        let digest = Sha1::new();
        let mut hmac = Hmac::new(digest, self.secret.as_bytes());
        hmac.input(payload.as_bytes());
        let expected_signature = hmac.result();

        let parts = signature.splitn(2, '=').collect::<Vec<_>>();
        let code = parts[1];

        crypto::util::fixed_time_eq(
            Self::bytes_to_hex(expected_signature.code()).as_bytes(),
            code.as_bytes(),
        )
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        const CHARS: &[u8] = b"0123456789abcdef";
        let mut v = Vec::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            v.push(CHARS[(byte >> 4) as usize]);
            v.push(CHARS[(byte & 0xf) as usize]);
        }

        unsafe { String::from_utf8_unchecked(v) }
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
            botname: std::env::var("BOTNAME").unwrap_or("ai-chan".to_owned()),
            github_api_key: std::env::var("GITHUB_API_KEY").unwrap_or_default(),
            secret: std::env::var("WEBHOOK_SECRET").unwrap_or_default(),
        }
    }
}
