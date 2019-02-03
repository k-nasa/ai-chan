use serde_derive::*;
use std::fs::*;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Owners {
    owners: Vec<String>,
}

#[cfg(test)]
mod test {
    #[test]
    fn from_toml_string() {
        let toml = r###"reviewrs = ["k-nasa"]"###;

        let owners = Owners {
            owners: vec!["k-nasa".to_string()],
        };

        assert_eq!(owners, toml::from_str(toml));
    }
}
