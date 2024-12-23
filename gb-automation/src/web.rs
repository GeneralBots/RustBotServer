use std::time::Duration;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol;
use chromiumoxide::page::Page;
use futures_util::StreamExt;
use gb_core::{Error, Result};
use tracing::instrument;

#[derive(Debug)]
pub struct Element {
    inner: chromiumoxide::element::Element,
}

pub struct WebAutomation {
    browser: Browser,
}

impl WebAutomation {
    #[instrument]
    pub async fn new() -> Result<Self> {
        let (browser, mut handler) = Browser::launch(BrowserConfig::default())
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
        let params = browser_protocol::page::CreateTarget::new();
        let page = self.browser.new_page(params)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(page)
    }

    #[instrument(skip(self))]
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        page.goto(url)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_element(&self, page: &Page, selector: &str) -> Result<Element> {
        let element = page.find_element(selector)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(Element { inner: element })
    }

    #[instrument(skip(self))]
    pub async fn screenshot(&self, page: &Page, _path: &str) -> Result<Vec<u8>> {
        let screenshot_params = browser_protocol::page::CaptureScreenshot::new();
        let data = page.screenshot(screenshot_params)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(data)
    }

    #[instrument(skip(self))]
    pub async fn wait_for_selector(&self, page: &Page, selector: &str) -> Result<()> {
        page.find_element(selector)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn wait_for_network_idle(&self, page: &Page) -> Result<()> {
        page.evaluate("() => new Promise(resolve => setTimeout(resolve, 1000))")
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    async fn automation() -> WebAutomation {
        WebAutomation::new().await.unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_navigation(automation: WebAutomation) -> Result<()> {
        let page = automation.new_page().await?;
        automation.navigate(&page, "https://example.com").await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_element_interaction(automation: WebAutomation) -> Result<()> {
        let page = automation.new_page().await?;
        automation.navigate(&page, "https://example.com").await?;
        let element = automation.get_element(&page, "h1").await?;
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_screenshot(automation: WebAutomation) -> Result<()> {
        let page = automation.new_page().await?;
        automation.navigate(&page, "https://example.com").await?;
        let screenshot = automation.screenshot(&page, "test.png").await?;
        Ok(())
    }
}