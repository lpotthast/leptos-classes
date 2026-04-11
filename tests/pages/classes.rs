use thirtyfour::WebDriver;

use crate::pages::BaseActions;

pub struct ClassesPage<'a> {
    pub driver: &'a WebDriver,
    pub base_url: &'a str,
}

impl BaseActions for ClassesPage<'_> {
    fn driver(&self) -> &WebDriver {
        self.driver
    }

    fn base_url(&self) -> &str {
        self.base_url
    }
}

impl ClassesPage<'_> {
    pub async fn goto(&self) -> anyhow::Result<()> {
        self.goto_path("/").await
    }

    pub async fn click(&self, id: &str) -> anyhow::Result<()> {
        self.click_element_with_id(id).await
    }
}
