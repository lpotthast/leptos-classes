pub mod test_classes;

use thirtyfour::WebDriver;

#[async_trait::async_trait]
pub trait UiTest {
    fn name(&self) -> String;

    async fn run(&self, driver: &WebDriver, base_url: &str) -> anyhow::Result<()>;
}
