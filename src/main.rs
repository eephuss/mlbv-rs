mod config;
mod session;

use crate::session::MlbSession;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut session = MlbSession::new()?;
    session.authenticate().await?;

    Ok(())
}
