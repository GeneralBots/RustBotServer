use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    env,
};

use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

// TODO: https://helpcenter.onlyoffice.com/docs/installation/docs-community-install-ubuntu.aspx


const INSTALL_DIR: &str = "/opt/gbo";
const TEMP_DIR: &str = "/tmp/gbotemp";

#[derive(Debug)]
struct Component {
    name: &'static str,
    bin_dir: &'static str,
    download_url: Option<&'static str>,
    archive_type: ArchiveType,
    binaries: Vec<&'static str>,
    config_files: Vec<ConfigFile>,
}

#[derive(Debug)]
struct ConfigFile {
    src_url: Option<&'static str>,
    src_path: Option<&'static str>,
    dest_name: &'static str,
}

#[derive(Debug)]
enum ArchiveType {
    TarGz,
    Zip,
    Binary,
}

pub fn doIt() -> io::Result<()> {
    // Define all components
    let components = [
        // Directory (Zitadel)
        Component {
            name: "zitadel",
            bin_dir: "directory",
            download_url: Some("https://github.com/zitadel/zitadel/releases/latest/download/zitadel_Linux_x86_64.tar.gz"),
            archive_type: ArchiveType::TarGz,
            binaries: vec!["zitadel"],
            config_files: vec![ConfigFile {
                src_url: None,
                src_path: Some("src/config/directory/zitadel.yaml"),
                dest_name: "zitadel.yaml",
            }],
        },
        // Mail (Stalwart)
        Component {
            name: "stalwart-mail",
            bin_dir: "mail",
            download_url: Some("https://github.com/stalwartlabs/mail-server/releases/latest/download/stalwart-linux-x86_64.tar.gz"),
            archive_type: ArchiveType::TarGz,
            binaries: vec!["stalwart-mail"],
            config_files: vec![ConfigFile {
                src_url: Some("https://raw.githubusercontent.com/stalwartlabs/mail-server/main/resources/config/config.toml"),
                src_path: None,
                dest_name: "config.toml",
            }],
        },
        // Tabular (PostgreSQL)
        Component {
            name: "postgresql",
            bin_dir: "tabular",
            download_url: Some("https://get.enterprisedb.com/postgresql/postgresql-14.10-1-linux-x64-binaries.tar.gz"),
            archive_type: ArchiveType::TarGz,
            binaries: vec!["postgres", "pg_ctl", "psql", "pg_dump", "pg_restore"],
            config_files: vec![],
        },
        // Object (MinIO)
        Component {
            name: "minio",
            bin_dir: "object",
            download_url: Some("https://dl.min.io/server/minio/release/linux-amd64/minio"),
            archive_type: ArchiveType::Binary,
            binaries: vec!["minio"],
            config_files: vec![],
        },
        // Webserver (Caddy)
        Component {
            name: "caddy",
            bin_dir: "webserver",
            download_url: Some("https://github.com/caddyserver/caddy/releases/latest/download/caddy_linux_amd64.tar.gz"),
            archive_type: ArchiveType::TarGz,
            binaries: vec!["caddy"],
            config_files: vec![ConfigFile {
                src_url: None,
                src_path: Some("src/config/webserver/Caddyfile"),
                dest_name: "Caddyfile",
            }],
        },
    ];

    // Create directories
    create_directories()?;

    // Install dependencies
    install_dependencies()?;

    // Create HTTP client
    let client = Client::new();

    // Process all components
    for component in components.iter() {
        install_component(&component, &client)?;
    }

    // Clean up temp directory
    fs::remove_dir_all(TEMP_DIR)?;

    println!("All binaries downloaded to {}", INSTALL_DIR);
    println!("Use the start-stop script to manually control all components");

    Ok(())
}

fn create_directories() -> io::Result<()> {
    println!("Creating directories...");

    // Main directories
    fs::create_dir_all(INSTALL_DIR)?;
    Command::new("chmod").args(["777", INSTALL_DIR]).status()?;
    fs::create_dir_all(TEMP_DIR)?;

    // Component directories
    let dirs = [
        "bin/bot", "bin/mail", "bin/tabular", "bin/object",
        "bin/directory", "bin/alm", "bin/webserver", "bin/meeting",
        "config/bot", "config/mail", "config/tabular", "config/object",
        "config/directory", "config/alm", "config/webserver", "config/meeting",
        "data/bot", "data/mail", "data/tabular", "data/object",
        "data/directory", "data/alm", "data/webserver", "data/meeting",
        "logs", "certs"
    ];

    for dir in dirs {
        fs::create_dir_all(format!("{}/{}", INSTALL_DIR, dir))?;
    }

    Ok(())
}

fn install_dependencies() -> io::Result<()> {
    println!("Installing system dependencies...");
    Command::new("apt-get").args(["update"]).status()?;
    Command::new("apt-get").args(["install", "-y",
        "apt-transport-https", "ca-certificates", "curl",
        "software-properties-common", "gnupg", "wget",
        "unzip", "tar", "postgresql-client", "redis-tools"
    ]).status()?;
    Ok(())
}

fn install_component(component: &Component, client: &Client) -> io::Result<()> {
    println!("Installing {}...", component.name);

    if let Some(url) = component.download_url {
        let temp_path = format!("{}/{}", TEMP_DIR, component.name);
        let target_dir = format!("{}/bin/{}", INSTALL_DIR, component.bin_dir);

        // Download the file
        download_file(client, url, &temp_path)?;

        match component.archive_type {
            ArchiveType::TarGz => {
                // Extract tar.gz archive
                let tar_gz = File::open(&temp_path)?;
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                archive.unpack(TEMP_DIR)?;

                // Move binaries to target directory
                for binary in &component.binaries {
                    let src = format!("{}/{}", TEMP_DIR, binary);
                    let dest = format!("{}/{}", target_dir, binary);
                    
                    if Path::new(&src).exists() {
                        fs::rename(&src, &dest)?;
                        set_executable(&dest)?;
                    } else {
                        // For PostgreSQL which has binaries in pgsql/bin/
                        let pg_src = format!("{}/pgsql/bin/{}", TEMP_DIR, binary);
                        if Path::new(&pg_src).exists() {
                            fs::rename(&pg_src, &dest)?;
                            set_executable(&dest)?;
                        }
                    }
                }
            },
            ArchiveType::Zip => {
                // Extract zip archive
                let file = File::open(&temp_path)?;
                let mut archive = ZipArchive::new(file)?;
                archive.extract(TEMP_DIR)?;

                // Move binaries to target directory
                for binary in &component.binaries {
                    let src = format!("{}/{}", TEMP_DIR, binary);
                    let dest = format!("{}/{}", target_dir, binary);
                    
                    if Path::new(&src).exists() {
                        fs::rename(&src, &dest)?;
                        set_executable(&dest)?;
                    }
                }
            },
            ArchiveType::Binary => {
                // Single binary - just move to target location
                let dest = format!("{}/{}", target_dir, component.name);
                fs::rename(&temp_path, &dest)?;
                set_executable(&dest)?;
            },
        }

        // Clean up downloaded file
        fs::remove_file(temp_path)?;
    }

    // Handle config files
    for config in &component.config_files {
        let config_dir = format!("{}/config/{}", INSTALL_DIR, component.bin_dir);
        let dest_path = format!("{}/{}", config_dir, config.dest_name);

        if let Some(url) = config.src_url {
            // Download config from URL
            download_file(client, url, &dest_path)?;
        } else if let Some(src_path) = config.src_path {
            // Copy config from local source (placeholder)
            println!("Would copy config from {} to {}", src_path, dest_path);
            // fs::copy(src_path, dest_path)?;
        }
    }

    println!("{} installed successfully!", component.name);
    Ok(())
}

fn download_file(client: &Client, url: &str, dest_path: &str) -> io::Result<()> {
    println!("Downloading {} to {}", url, dest_path);
    
    let mut response = client.get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download file: HTTP {}", response.status())
        ));
    }

    let mut dest_file = File::create(dest_path)?;
    response.copy_to(&mut dest_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

fn set_executable(path: &str) -> io::Result<()> {
    Command::new("chmod")
        .args(["+x", path])
        .status()?;
    Ok(())
}