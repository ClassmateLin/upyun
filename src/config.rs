use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password: String,
}

// 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub account: Vec<Account>,
}

impl Config {
    // 从配置文件中读取
    pub async fn from_file(path: Option<String>) -> Result<Self> {
        let path = match path {
            Some(p) => p,
            None => String::from("config.toml"),
        };

        let conf_string = fs::read_to_string(path)?;

        Ok(toml::from_str::<Config>(&conf_string)?)
    }
}
