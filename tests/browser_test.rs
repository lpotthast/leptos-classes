//! Browser integration tests for the Leptos class attribute lifecycle.
#![cfg(not(target_arch = "wasm32"))]

mod pages;
mod ui_tests;

use std::time::Duration;

use browser_test::thirtyfour::ChromiumLikeCapabilities;
use browser_test::{
    BrowserTestFailurePolicy, BrowserTestParallelism, BrowserTestRunner, BrowserTestVisibility,
    BrowserTests, BrowserTimeouts, PauseConfig,
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
        .with_chrome_capabilities(|caps| {
            // `--no-sandbox` disables Chrome's child-process sandboxing. User-mode tarball
            // extraction can't set the setuid-root bit on `chrome_sandbox` (the privileged helper
            // Chrome execs to install the sandbox), and CI kernels often also restrict
            // unprivileged user namespaces. Without either layer, Chrome exits before chromedriver
            // opens a session.
            caps.add_arg("--no-sandbox")?;

            // `--disable-dev-shm-usage` makes Chrome place its IPC shared-memory segments under
            // `/tmp` instead of `/dev/shm`. CI runners typically expose `/dev/shm` tiny tmpfs,
            // which Chrome can exhaust during session startup.
            caps.add_arg("--disable-dev-shm-usage")?;
            Ok(())
        })
        .with_test_parallelism(BrowserTestParallelism::Sequential)
        .with_failure_policy(BrowserTestFailurePolicy::RunAll)
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
