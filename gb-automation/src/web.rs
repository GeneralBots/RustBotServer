use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::element::Element;
use chromiumoxide::page::Page;
use futures_util::StreamExt;
use gb_core::{Error, Result};
use tracing::instrument;

pub struct WebAutomation {
    browser: Browser,
}

impl WebAutomation {
    #[instrument]
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .build()
            .map_err(|e| Error::internal(e.to_string()))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    tracing::error!("Browser handler error: {}", e);
                }
            }
        });

        Ok(Self { browser })
    }

    #[instrument(skip(self))]
    pub async fn new_page(&self) -> Result<Page> {
        let params = chromiumoxide::cdp::browser_protocol::target::CreateTarget::new()
            .url("about:blank");
        
        self.browser.new_page(params)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        page.goto(url)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn get_element(&self, page: &Page, selector: &str) -> Result<Element> {
        page.find_element(selector)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn screenshot(&self, page: &Page, _path: &str) -> Result<Vec<u8>> {
        let params = chromiumoxide::cdp::browser_protocol::page::CaptureScreenshot::new();
        page.screenshot(params)
            .await
            .map_err(|e| Error::internal(e.to_string()))
    }

    #[instrument(skip(self))]
    pub async fn wait_for_selector(&self, page: &Page, selector: &str) -> Result<()> {
        page.find_element(selector)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }
}
