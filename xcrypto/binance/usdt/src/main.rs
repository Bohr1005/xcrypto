mod trade;

use binance::*;
use clap::Parser;
use log::{error, info};
use logger::*;
use serde::Deserialize;
use std::sync::Arc;
use trade::UsdtTrade;
use xcrypto::rest::Rest;

#[derive(Debug, Deserialize)]
struct Config {
    apikey: String,
    pem: String,
    local: String,
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long, help = "Config path")]
    config: String,
    #[arg(short, long, default_value_t = Level::Info)]
    level: Level,
}

impl Args {
    pub fn load(&self) -> anyhow::Result<Config> {
        info!("Load config from {}", self.config);
        let buf = std::fs::read_to_string(self.config.clone())?;
        let config: Config = native_json::parse(&buf)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = args.load()?;

    let path = std::env::current_exe()?;
    let filename = match path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => "unknown".into(),
    };

    let _logger = init(Some(format!("log/{}", filename)), args.level);

    let app = Application::new(&config.local).await?;
    let market = Market::new("wss://fstream.binance.com/ws".into()).await?;

    let rest = Arc::new(Rest::new(
        "https://fapi.binance.com",
        &config.apikey,
        &config.pem,
        3000,
    )?);

    let account = Account::new(
        "wss://fstream.binance.com/ws",
        "/fapi/v1/listenKey",
        rest.clone(),
    )
    .await?;
    let trade = UsdtTrade::new(rest.clone(), account).await?;

    if let Err(e) = app.keep_running(market, trade).await {
        error!("{}", e);
    }

    Ok(())
}
