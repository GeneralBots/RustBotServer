use gb_core::{Result, Error};
use async_recursion::async_recursion;
use chromiumoxide::{
    Browser, BrowserConfig, 
    cdp::browser_protocol::page::ScreenshotFormat,
    Page,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing::{instrument, error};

pub struct WebAutomation {
    browser: Arc<Browser>,
    pages: Arc<Mutex<Vec<Page>>>,
}

impl WebAutomation {
    #[instrument]
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .with_head()
            .window_size(1920, 1080)
            .build()?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| Error::internal(format!("Failed to launch browser: {}", e)))?;

        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    error!("Browser handler error: {}", e);
                }
            }
        });

        Ok(Self {
            browser: Arc::new(browser),
            pages: Arc::new(Mutex::new(Vec::new())),
        })
    }

    #[instrument(skip(self))]
    pub async fn new_page(&self) -> Result<Page> {
        let page = self.browser.new_page()
            .await
            .map_err(|e| Error::internal(format!("Failed to create page: {}", e)))?;

        let mut pages = self.pages.lock().await;
        pages.push(page.clone());

        Ok(page)
    }

    #[instrument(skip(self))]
    pub async fn navigate(&self, page: &Page, url: &str) -> Result<()> {
        page.goto(url)
            .await
            .map_err(|e| Error::internal(format!("Failed to navigate: {}", e)))?;

        page.wait_for_navigation()
            .await
            .map_err(|e| Error::internal(format!("Failed to wait for navigation: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_element(&self, page: &Page, selector: &str) -> Result<Element> {
        let element = page.find_element(selector)
            .await
            .map_err(|e| Error::internal(format!("Failed to find element: {}", e)))?;

        Ok(Element { inner: element })
    }

    #[instrument(skip(self))]
    pub async fn click(&self, element: &Element) -> Result<()> {
        element.inner.click()
            .await
            .map_err(|e| Error::internal(format!("Failed to click: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn type_text(&self, element: &Element, text: &str) -> Result<()> {
        element.inner.type_str(text)
            .await
            .map_err(|e| Error::internal(format!("Failed to type text: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn screenshot(&self, page: &Page, path: &str) -> Result<Vec<u8>> {
        let screenshot = page.screenshot(ScreenshotFormat::PNG, None, true)
            .await
            .map_err(|e| Error::internal(format!("Failed to take screenshot: {}", e)))?;

        Ok(screenshot)
    }

    #[instrument(skip(self))]
    pub async fn wait_for_selector(&self, page: &Page, selector: &str) -> Result<()> {
        page.wait_for_element(selector)
            .await
            .map_err(|e| Error::internal(format!("Failed to wait for selector: {}", e)))?;

        Ok(())
    }

    #[instrument(skip(self))]
    #[async_recursion]
    pub async fn wait_for_network_idle(&self, page: &Page) -> Result<()> {
        let mut retry_count = 0;
        let max_retries = 10;

        while retry_count < max_retries {
            if page.wait_for_network_idle(Duration::from_secs(5))
                .await
                .is_ok()
            {
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Err(Error::internal("Network did not become idle".to_string()))
    }
}

pub struct Element {
    inner: chromiumoxide::Element,
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
        
        let title = page.title()
            .await
            .map_err(|e| Error::internal(format!("Failed to get title: {}", e)))?;
            
        assert!(title.contains("Example"));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_element_interaction(automation: WebAutomation) -> Result<()> {
        let page = automation.new_page().await?;
        automation.navigate(&page, "https://example.com").await?;

        let element = automation.get_element(&page, "h1").await?;
        automation.click(&element).await?;

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
