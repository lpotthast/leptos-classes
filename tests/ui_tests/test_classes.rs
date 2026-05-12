use std::borrow::Cow;

use browser_test::{BrowserTest, async_trait, thirtyfour::WebDriver};
use leptos_browser_test::Report;

use crate::pages::{BaseActions, classes::ClassesPage};

pub struct ClassesTests {}

#[async_trait]
impl BrowserTest<str> for ClassesTests {
    fn name(&self) -> Cow<'_, str> {
        "classes_tests".into()
    }

    async fn run(&self, driver: &WebDriver, base_url: &str) -> Result<(), Report> {
        let page = ClassesPage { driver, base_url };
        page.goto().await?;

        assert_static_target(&page).await?;
        assert_hydrate_target(&page).await?;
        assert_reactive_target(&page).await?;
        assert_toggle_target(&page).await?;
        assert_drilling_target(&page).await?;
        assert_transition_target(&page).await?;
        assert_managed_target(&page).await?;
        assert_empty_target(&page).await?;

        Ok(())
    }
}

async fn assert_static_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("static-target", "foo bar baz").await
}

async fn assert_hydrate_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("hydrate-target", "base fresh").await
}

async fn assert_reactive_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("reactive-target", "base highlighted")
        .await?;

    page.click("reactive-clear").await?;
    page.wait_for_class("reactive-target", "base").await?;

    page.click("reactive-restore").await?;
    page.wait_for_class("reactive-target", "base highlighted")
        .await
}

async fn assert_toggle_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("toggle-target", "base active").await?;

    page.click("toggle-set-false").await?;
    page.wait_for_class("toggle-target", "base inactive")
        .await?;

    page.click("toggle-set-true").await?;
    page.wait_for_class("toggle-target", "base active").await
}

async fn assert_drilling_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("drilling-target", "layer-root layer-mid layer-leaf")
        .await
}

async fn assert_transition_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("transition-target", "base active")
        .await?;

    page.click("transition-set-false").await?;
    page.wait_for_class("transition-target", "base inactive")
        .await?;

    page.click("transition-set-true").await?;
    page.wait_for_class("transition-target", "base active")
        .await?;

    page.click("transition-freeze-static").await?;
    page.wait_for_class("transition-target", "base frozen")
        .await?;

    // While frozen, mutating the reactive signal must not leak through.
    page.click("transition-set-false").await?;
    page.wait_for_class("transition-target", "base frozen")
        .await?;

    page.click("transition-enable-reactive").await?;
    page.wait_for_class("transition-target", "base inactive")
        .await?;

    page.click("transition-set-true").await?;
    page.wait_for_class("transition-target", "base active")
        .await
}

async fn assert_managed_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("managed-target", "managed active")
        .await?;

    // Stomp on the attribute from outside the library.
    page.click("managed-mutate").await?;
    page.wait_for_class("managed-target", "externally-mutated")
        .await?;

    // The default signal value for `active` is `true`. Setting the signal to `true` again here
    // does not **change** the signal value and is *not* expected to reconcile: `Classes` treats its
    // in-memory `current` as authoritative for the DOM and external mutations are out of contract.
    page.click("managed-set-active").await?;
    page.wait_for_class("managed-target", "externally-mutated")
        .await?;

    // Only the next value-changing update reconciles the attribute.
    page.click("managed-set-inactive").await?;
    page.wait_for_class("managed-target", "managed").await?;

    page.click("managed-set-active").await?;
    page.wait_for_class("managed-target", "managed active")
        .await
}

async fn assert_empty_target(page: &ClassesPage<'_>) -> Result<(), Report> {
    page.wait_for_class("empty-target", "ephemeral").await?;

    page.click("empty-clear").await?;
    page.wait_for_no_class_attribute("empty-target").await?;

    page.click("empty-restore").await?;
    page.wait_for_class("empty-target", "ephemeral").await
}
