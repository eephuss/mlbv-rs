mod config;
mod session;

use crate::config::AppConfig;
use crate::session::MlbSession;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::load()?;

    let session = MlbSession::new()?
        .authenticate(&cfg.credentials.username, &cfg.credentials.password)
        .await?
        .fetch_okta_code()
        .await?
        .exchange_tokens()
        .await?;

    println!("{:?}", session.state.okta_tokens);

    Ok(())
}
