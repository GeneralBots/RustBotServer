use crate::services::{zitadel, stalwart, minio, postgresql, nginx};
use dotenv::dotenv;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ServiceManager {
    services: Vec<Box<dyn Service>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        dotenv().ok();
        ServiceManager {
            services: vec![
                Box::new(zitadel::Zitadel::new()),
                Box::new(stalwart::Stalwart::new()),
                Box::new(minio::MinIO::new()),
                Box::new(postgresql::PostgreSQL::new()),
                Box::new(nginx::NGINX::new()),
            ],
        }
    }

    pub fn start(&mut self) {
        for service in &mut self.services {
            service.start().unwrap();
        }
    }

    pub fn stop(&mut self) {
        for service in &mut self.services {
            service.stop().unwrap();
        }
    }

    pub fn run(&mut self) {
        self.start();
        let running = Arc::new(Mutex::new(true));
        let running_clone = Arc::clone(&running);

        ctrlc::set_handler(move || {
            println!("Exiting service manager...");
            let mut running = running_clone.lock().unwrap();
            *running = false;
        })
        .expect("Failed to set Ctrl+C handler.");

        while *running.lock().unwrap() {
            thread::sleep(Duration::from_secs(1));
        }

        self.stop();
    }
}

pub trait Service {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
}
