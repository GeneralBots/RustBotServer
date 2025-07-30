use crate::services::utils;
use log::debug;
use std::env;
use std::env::temp_dir;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::process::Command;
use std::sync::Arc;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio::fs;
use tokio::sync::Semaphore;

pub struct BrowserSetup {
    pub brave_path: String,
    pub chromedriver_path: String,
}

pub struct BrowserPool {
    webdriver_url: String,
    semaphore: Semaphore,
    brave_path: String,
}

impl BrowserPool {
    pub fn new(webdriver_url: String, max_concurrent: usize, brave_path: String) -> Self {
        Self {
            webdriver_url,
            semaphore: Semaphore::new(max_concurrent),
            brave_path,
        }
    }

    pub async fn with_browser<F, T>(&self, f: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnOnce(
                WebDriver,
            )
                -> Pin<Box<dyn Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        let _permit = self.semaphore.acquire().await?;

        let mut caps = DesiredCapabilities::chrome();
        caps.set_binary(&self.brave_path)?;
        caps.add_chrome_arg("--headless=new")?;
        caps.add_chrome_arg("--disable-gpu")?;
        caps.add_chrome_arg("--no-sandbox")?;

        let driver = WebDriver::new(&self.webdriver_url, caps).await?;

        // Execute user function
        let result = f(driver).await;

        result
    }
}

impl BrowserSetup {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Check for Brave installation
        let brave_path = Self::find_brave().await?;

        // Check for chromedriver
        let chromedriver_path = Self::setup_chromedriver().await?;

        Ok(Self {
            brave_path,
            chromedriver_path,
        })
    }

    async fn find_brave() -> Result<String, Box<dyn std::error::Error>> {
        let possible_paths = vec![
            // Windows
            String::from(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"),
            // macOS
            String::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            // Linux
            String::from("/usr/bin/brave-browser"),
            String::from("/usr/bin/brave"),
        ];

        for path in possible_paths {
            if fs::metadata(&path).await.is_ok() {
                return Ok(path);
            }
        }

        Err("Brave browser not found. Please install Brave first.".into())
    }

    async fn setup_chromedriver() -> Result<String, Box<dyn std::error::Error>> {
        let mut chromedriver_path = env::current_exe()?.parent().unwrap().to_path_buf();
        chromedriver_path.push("chromedriver");

        // Check if chromedriver exists
        if fs::metadata(&chromedriver_path).await.is_err() {
            println!("Downloading chromedriver...");

            // Note: This URL structure is outdated. Consider using Chrome for Testing endpoints
            let (base_url, platform) =
                match (cfg!(target_os = "windows"), cfg!(target_arch = "x86_64")) {
                    (true, true) => (
                        "https://chromedriver.storage.googleapis.com/114.0.5735.90",
                        "win32",
                    ),
                    (false, true) if cfg!(target_os = "macos") => (
                        "https://chromedriver.storage.googleapis.com/114.0.5735.90",
                        "mac64",
                    ),
                    (false, true) => (
                        "https://chromedriver.storage.googleapis.com/114.0.5735.90",
                        "linux64",
                    ),
                    _ => return Err("Unsupported platform".into()),
                };

            let download_url = format!("{}/chromedriver_{}.zip", base_url, platform);

            let mut zip_path = temp_dir();
            zip_path.push("chromedriver.zip");

            utils::download_file(&download_url, &zip_path.to_str().unwrap()).await?;

            let extract_result = utils::extract_zip_recursive(&zip_path, &chromedriver_path);
            if let Err(e) = extract_result {
                debug!("Error extracting ZIP: {}", e);
            }
            // Clean up zip file
            let _ = fs::remove_file(&zip_path).await;

            if cfg!(target_os = "windows") {
            chromedriver_path.push("chromedriver.exe");
        } else {
            chromedriver_path.push("chromedriver");
        }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&chromedriver_path).await?.permissions();
                perms.set_mode(0o755); // Make executable
                fs::set_permissions(&chromedriver_path, perms).await?;
            }
        }

        Ok(chromedriver_path.to_string_lossy().to_string())
    }
}

// Modified BrowserPool initialization
pub async fn initialize_browser_pool() -> Result<Arc<BrowserPool>, Box<dyn std::error::Error>> {
    let setup = BrowserSetup::new().await?;

    // Start chromedriver process if not running
    if !is_process_running("chromedriver").await {
        Command::new(&setup.chromedriver_path)
            .arg("--port=9515")
            .spawn()?;

        // Give chromedriver time to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(Arc::new(BrowserPool::new(
        "http://localhost:9515".to_string(),
        5, // Max concurrent browsers
        setup.brave_path,
    )))
}

async fn is_process_running(name: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(name))
            .unwrap_or(false)
    } else {
        Command::new("pgrep")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
