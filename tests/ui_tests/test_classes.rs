use std::time::Duration;

use thirtyfour::{TimeoutConfiguration, WebDriver};

use crate::{
    pages::{BaseActions, classes::ClassesPage},
    ui_tests::UiTest,
};

pub struct ClassesTests {}

#[async_trait::async_trait]
impl UiTest for ClassesTests {
    fn name(&self) -> String {
        "classes_tests".to_string()
    }

    async fn run(&self, driver: &WebDriver, base_url: &str) -> anyhow::Result<()> {
        let mut timeouts = TimeoutConfiguration::default();
        timeouts.set_implicit(Some(Duration::from_secs(3)));
        driver.update_timeouts(timeouts).await?;

        let page = ClassesPage { driver, base_url };
        page.goto().await?;

        page.wait_for_class("managed-target", "managed active")
            .await?;
        page.click("managed-mutate").await?;
        page.wait_for_class("managed-target", "managed active external")
            .await?;
        page.click("managed-set-same-active").await?;
        page.wait_for_class("managed-target", "managed active")
            .await?;
        page.click("managed-set-inactive").await?;
        page.wait_for_class("managed-target", "managed").await?;
        page.click("managed-set-active").await?;
        page.wait_for_class("managed-target", "managed active")
            .await?;

        page.wait_for_class("transition-target", "base active")
            .await?;
        page.click("transition-freeze-static").await?;
        page.wait_for_class("transition-target", "base frozen")
            .await?;
        page.click("transition-set-false").await?;
        page.wait_for_class("transition-target", "base frozen")
            .await?;
        page.click("transition-enable-reactive").await?;
        page.wait_for_class("transition-target", "base inactive")
            .await?;
        page.click("transition-set-true").await?;
        page.wait_for_class("transition-target", "base active")
            .await?;

        page.wait_for_class("empty-target", "ephemeral").await?;
        page.click("empty-set-false").await?;
        page.wait_for_class("empty-target", "").await?;
        page.click("empty-set-true").await?;
        page.wait_for_class("empty-target", "ephemeral").await?;

        Ok(())
    }
}
