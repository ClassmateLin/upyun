use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, Utc};
use log::info;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Account;

/// User info
#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    username: String,
    user_id: u64,
}

pub struct App {
    client: Client,
    base_url: String,
    user: User,
}

/// 任务详细信息
#[derive(Serialize, Deserialize, Clone, Debug)]
struct TaskItem {
    id: u32,
    cycle: String,
    mileage: u8,
    description: String,
    count: u8,
    event_id: String,
    finish: bool,
}

/// 任务
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Tasks {
    unlimited: Vec<TaskItem>,
}

/// 用户已获得的奖品
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Awards {
    alcatraz: Vec<Value>,
    crystal: Vec<Value>,
    pyramid: Vec<Value>,
    volcanic: Vec<Value>,
    waterfall: Vec<Value>,
    wishes: Vec<Value>,
}

impl App {
    pub async fn new(account: &Account) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.append(
            "origin",
            HeaderValue::from_str("https://www.upyun.com").unwrap(),
        );

        headers.append(
            "referer",
            HeaderValue::from_str("https://www.upyun.com/onePiece").unwrap(),
        );

        let base_url = "https://console.upyun.com".to_string();

        let login_url = format!("{}/accounts/signin/", base_url);
        let client = Client::builder()
        .default_headers(headers.clone())
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Mobile Safari/537.36")
        .build()
        .unwrap();

        let data = client
            .post(&login_url)
            .json(&json!({
                "username": account.username,
                "password": account.password,
                "from": "https://www.upyun.com/"
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        let user = serde_json::from_value::<User>(data["user"].clone())?;

        let resp = client
            .post(login_url)
            .json(&json!({
                "username": account.username,
                "password": account.password,
                "from": "https://www.upyun.com/"
            }))
            .send()
            .await?;

        let mut sid = String::from("");

        for cookie in resp.cookies() {
            if cookie.name().eq("sid") {
                sid = cookie.value().to_string();
            }
        }

        if sid.is_empty() {
            return Err(anyhow!("获取sid失败!"));
        }

        headers.append(
            "cookie",
            HeaderValue::from_str(&format!("sid={};", sid)).unwrap(),
        );

        let client = Client::builder()
        .default_headers(headers)
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Mobile Safari/537.36")
        .build()
        .unwrap();

        Ok(Self {
            client,
            base_url,
            user,
        })
    }

    fn user(&self) -> String {
        format!("{:?}({:?})", self.user.username, self.user.user_id).replace('"', "")
    }

    /// 完成任务
    /// event_id: 任务ID
    async fn finish_task(&self, event_id: String) -> Result<bool> {
        let url = format!("{}/activity/tasks", self.base_url);
        if let Ok(data) = self
            .client
            .post(&url)
            .header("x-token", "7c4ee7db-d67e-4912-b72b-9e2439352716")
            .json(&json!({
                "accountName": self.user.username,
                "event_id": event_id
            }))
            .send()
            .await?
            .json::<Value>()
            .await
        {
            return Ok(data["result"].as_bool().unwrap_or(false));
        }
        Ok(false)
    }

    /// 获取任务列表并完成任务
    async fn do_tasks(&self) -> Result<()> {
        let url = format!("{}/activity/tasks", self.base_url);

        let tasks = match self.client.get(&url).send().await?.json::<Tasks>().await {
            Ok(data) => data,
            Err(_) => {
                info!("{}, 获取任务列表失败!", self.user());
                return Ok(());
            }
        };

        for item in tasks.unlimited {
            if item.finish {
                info!(
                    "{:?}, 今日已完成任务《{:?}》!",
                    self.user(),
                    item.description
                );
                continue;
            }
            let bool = self.finish_task(item.event_id).await?;
            match bool {
                true => info!(
                    "{:?}, 完成任务《{}》, 获得海里数:{:?}!",
                    self.user(),
                    item.description,
                    item.mileage
                ),
                false => info!("{:?}, 无法完成任务《{}》!", self.user(), item.description),
            };
            sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    /// 获取里程数信息, 并航行
    pub async fn mileage(&self) -> Result<()> {
        let url = format!("{}/activity/mileage", self.base_url);
        let data = match self.client.get(&url).send().await?.json::<Value>().await {
            Ok(data) => data,
            Err(_) => {
                info!("{}, 获取海里数信息失败!", self.user());
                return Ok(());
            }
        };
        let can_use_mileage = data["data"]["canUse"].as_u64().unwrap_or(0);
        let used_mileage = data["data"]["used"].as_u64().unwrap_or(0);

        info!(
            "{:?}, 当前已航行里程数:{:?}, 剩余里程数:{:?}!",
            self.user(),
            used_mileage,
            can_use_mileage
        );

        let times = can_use_mileage / 10;

        info!("{:?}, 当前可航行次数为:{}!", self.user(), times);

        let url = format!("{}/activity/mileage/use", self.base_url);
        for _ in 0..times {
            let data = match self.client.post(&url).send().await?.json::<Value>().await {
                Ok(data) => data,
                Err(_) => continue,
            };
            let result = data["result"].as_bool().unwrap_or(false);
            if !result {
                info!("{}, 航行失败, {}!", self.user(), data);
                break;
            } else {
                info!("{}, 航行成功!", self.user())
            }
            sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    pub async fn signin(&self) -> Result<()> {
        let url = format!("{}/activity/signin", self.base_url);
        let data = self.client.get(&url).send().await?.json::<Value>().await?;

        let mut can_signin = true;
        let items = Vec::new();
        let items = data["items"].as_array().unwrap_or(&items);
        let utc: DateTime<Utc> = Utc::now();
        let today = utc
            .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
            .format("%Y-%m-%d")
            .to_string();
        let continuous = data["continuous"].as_u64().unwrap_or(0);

        for item in items {
            let date = item["date"].to_string().replace('"', "");
            let is_signin = item["signin"].as_bool().unwrap_or(false);
            if date.eq(&today) && is_signin {
                can_signin = false;
            }
        }

        if !can_signin {
            info!(
                "{:?}, 今日已签到, 已连续签到{:?}天!",
                self.user(),
                continuous
            );
            return Ok(());
        }

        let data = self.client.post(url).send().await?.json::<Value>().await?;

        if let true = data["result"].as_bool().unwrap_or(false) {
            info!("{}, 签到成功!", self.user());
        } else {
            info!("{}, 签到失败!", self.user());
        }

        Ok(())
    }

    /// 获取奖品信息, 目前不知道prizeMap中的数据结构, 暂时只统计奖品个数
    async fn get_award_info(&self) -> Result<()> {
        let mut award_count = 0;
        let url = format!("{}/activity/user/award", self.base_url);
        if let Ok(data) = self.client.get(url).send().await?.json::<Value>().await {
            let awards = Map::<String, Value>::new();
            let awards = data["prizeMap"].as_object().unwrap_or(&awards);
            for (_, val) in awards {
                let items = Vec::new();
                let items = val.as_array().unwrap_or(&items);
                if !items.is_empty() {
                    award_count += 1;
                }
            }
            info!("{:?}, 有{:?}个奖品可以领取!", self.user(), award_count);
        } else {
            info!("{:?}, 获取奖品信息失败!", self.user());
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            "Login successful, Username:{}, User id:{}!",
            self.user.username, self.user.user_id
        );
        let _ = self.signin().await;
        let _ = self.do_tasks().await;
        let _ = self.mileage().await;
        let _ = self.get_award_info().await;
        Ok(())
    }
}
