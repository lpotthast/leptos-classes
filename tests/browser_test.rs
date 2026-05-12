//! Browser integration tests for the Leptos class attribute lifecycle.
#![cfg(not(target_arch = "wasm32"))]

mod pages;
mod ui_tests;

use std::time::Duration;

use browser_test::{
    BrowserTestRunner, BrowserTestVisibility, BrowserTests, BrowserTimeouts, PauseConfig,
};
use leptos_browser_test::{LeptosTestAppConfig, Report};

#[tokio::test(flavor = "multi_thread")]
async fn browser_tests() -> Result<(), Report> {
    tracing_subscriber::fmt().init();

    let app = LeptosTestAppConfig::new("testing/test-app")
        .with_app_name("leptos-classes test app")
        .start()
        .await?;

    BrowserTestRunner::new()
        .with_visibility(BrowserTestVisibility::from_env())
        .with_pause(PauseConfig::from_env())
        .with_timeouts(
            BrowserTimeouts::builder()
                .implicit_wait_timeout(Duration::from_secs(3))
                .build(),
        )
        .run(
            app.base_url(),
            BrowserTests::new().with(ui_tests::test_classes::ClassesTests {}),
        )
        .await?;

    Ok(())
}
