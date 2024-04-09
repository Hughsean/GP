
use std::fs;





#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    pub listen: String,
    pub cert_path: String,
    pub key_path: String,
}


impl Config {
   pub fn new(config: Option<&str>) -> Self {
        let default = Default::default();
        match config {
            Some(path) => match fs::read_to_string(path) {
                Ok(str) => toml::from_str::<Config>(&str).unwrap_or(default),
                Err(_) => default,
            },
            None => default,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:12345".into(),
            cert_path: "cert/cert.der".into(),
            key_path: "cert/key.der".into(),
        }
    }
}
