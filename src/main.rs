use anyhow::Result;
use futures::future::join_all;
use log::info;
use std::env;
use upyun::{app::App, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init_timed();

    let config = match Config::from_file(None).await {
        Ok(val) => val,
        Err(_) => {
            info!("请检查配置文件!");
            return Ok(());
        }
    };

    let mut handlers = Vec::new();

    for account in config.account {
        let handler = tokio::spawn(async move {
            if let Ok(app) = App::new(&account).await {
                let _ = app.run().await;
            } else {
                info!("账号或者密码错误: {:?}", account);
            }
        });
        handlers.push(handler);
    }

    let _ = join_all(handlers).await;

    Ok(())
}
