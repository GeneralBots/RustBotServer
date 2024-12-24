use chromiumoxide::{Browser, Element};
use chromiumoxide::page::Page;
use chromiumoxide::browser::BrowserConfig;
use futures_util::StreamExt;
use gb_core::{Error, Result};
use std::time::Duration;

pub struct WebAutomation {
    browser: Browser,
}

impl WebAutomation {
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .build()
            .map_err(|e| Error::internal(e.to_string()))?;
            
        let (browser, handler) = Browser::launch(config)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        // Spawn the handler in the background
        tokio::spawn(async move {
            handler.for_each(|_| async {}).await;
        });

        Ok(Self { browser })
    }

    pub async fn new_page(&self) -> Result<Page> {
        self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        page.goto(url)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }

    pub async fn take_screenshot(&self, page: &Page) -> Result<Vec<u8>> {
        let params = chromiumoxide::page::ScreenshotParams::builder().build();
            
        page.screenshot(params)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    pub async fn find_element(&self, page: &Page, selector: &str, timeout: Duration) -> Result<Element> {
        tokio::time::timeout(
            timeout,
            page.find_element(selector)
        )
        .await
        .map_err(|_| Error::internal("Timeout waiting for element"))?
        .map_err(|e| Error::internal(e.to_string()))
    }
}