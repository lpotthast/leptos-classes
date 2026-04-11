#![cfg(not(target_arch = "wasm32"))]

mod pages;
mod test_app;
mod ui_tests;

use std::{sync::Once, time::Duration};

use anyhow::{Context, Result, bail};
use chrome_for_testing_manager::{
    Channel, Chromedriver, PortRequest, Session, SessionError, VersionRequest,
};
use thirtyfour::{By, ChromeCapabilities, ChromiumLikeCapabilities, WebDriver};
use tokio::time::sleep;
use ui_tests::UiTest;

const DELAY_TEST_EXECUTION: bool = false;
static INIT_TRACING: Once = Once::new();

fn init_tracing() {
    INIT_TRACING.call_once(|| {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    });
}

async fn class_of(driver: &WebDriver, id: &str) -> Result<String> {
    driver
        .find(By::Id(id.to_owned()))
        .await
        .with_context(|| format!("failed to find element #{id}"))?
        .attr("class")
        .await
        .with_context(|| format!("failed to read class attribute from #{id}"))?
        .with_context(|| format!("element #{id} has no class attribute"))
}

async fn wait_for_class(driver: &WebDriver, id: &str, expected: &str) -> Result<()> {
    let timeout = Duration::from_secs(20);
    let poll_interval = Duration::from_millis(100);
    let deadline = std::time::Instant::now() + timeout;
    let mut last_seen = String::new();

    while std::time::Instant::now() < deadline {
        if let Ok(actual) = class_of(driver, id).await {
            if actual == expected {
                return Ok(());
            }
            last_seen = actual;
        }
        sleep(poll_interval).await;
    }

    bail!("timed out waiting for #{id} class {expected:?}; last seen {last_seen:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn browser_tests() -> Result<()> {
    init_tracing();

    let frontend = test_app::start_frontend().await?;
    let base_url = frontend.base_url.clone();

    if DELAY_TEST_EXECUTION {
        tracing::info!("Continue with tests? y/n");
        let mut buf = String::new();
        loop {
            buf.clear();
            let input = std::io::stdin().read_line(&mut buf);
            if input.is_ok() {
                match buf.trim() {
                    "y" => break,
                    "n" => return Ok(()),
                    _ => {}
                }
            }
            if let Err(err) = input {
                tracing::error!("Error reading input: {err:?}");
                return Err(err.into());
            }
        }
    }

    let tests: Vec<Box<dyn UiTest>> = vec![Box::new(ui_tests::test_classes::ClassesTests {})];

    tracing::info!("Starting webdriver...");
    let chromedriver =
        Chromedriver::run(VersionRequest::LatestIn(Channel::Stable), PortRequest::Any).await?;

    #[allow(clippy::redundant_closure_for_method_calls)]
    let test_result = async {
        for test in tests {
            chromedriver
                .with_custom_session(
                    |caps: &mut ChromeCapabilities| {
                        if std::env::var("BROWSER_TEST_VISIBLE").is_ok() {
                            caps.unset_headless()?;
                        }
                        Ok(())
                    },
                    async |session: &Session| {
                        tracing::info!("Executing test: {}", test.name());
                        test.run(session, &base_url)
                            .await
                            .map_err(|err| SessionError::Panic {
                                reason: err.to_string(),
                            })?;

                        wait_for_class(session, "managed-target", "managed active")
                            .await
                            .map_err(|err| SessionError::Panic {
                                reason: err.to_string(),
                            })?;

                        Ok(())
                    },
                )
                .await?;
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;

    chromedriver.terminate().await?;
    drop(frontend);
    test_result
}
