pub mod classes;
use std::time::Duration;

use anyhow::{Result, bail};
use thirtyfour::{By, WebDriver};
use tokio::time::sleep;

pub trait BaseActions {
    fn driver(&self) -> &WebDriver;
    fn base_url(&self) -> &str;

    async fn goto_path(&self, path: &str) -> Result<()> {
        let url = format!("{}{path}", self.base_url());
        self.driver().goto(&url).await?;
        Ok(())
    }

    async fn click_element_with_id(&self, id: &str) -> Result<()> {
        tracing::info!("Click element with id '{id}'.");
        let element = self.driver().find(By::Id(id)).await?;
        element.click().await?;
        Ok(())
    }

    async fn class_of(&self, id: &str) -> Result<String> {
        let element = self.driver().find(By::Id(id)).await?;
        Ok(element.attr("class").await?.unwrap_or_default())
    }

    async fn wait_for_class(&self, id: &str, expected: &str) -> Result<()> {
        let timeout = Duration::from_secs(20);
        let poll_interval = Duration::from_millis(100);
        let deadline = std::time::Instant::now() + timeout;
        let mut last_seen = String::new();

        while std::time::Instant::now() < deadline {
            if let Ok(actual) = self.class_of(id).await {
                if actual == expected {
                    return Ok(());
                }
                last_seen = actual;
            }
            sleep(poll_interval).await;
        }

        bail!("timed out waiting for #{id} class {expected:?}; last seen {last_seen:?}");
    }
}
