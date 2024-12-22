use rand::{distributions::Alphanumeric, Rng};
use std::time::Duration;

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|_| rand::random::<u8>()).collect()
}

pub async fn exponential_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut retries = 0;
    let mut delay = initial_delay;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if retries >= max_retries {
                    return Err(error);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
                retries += 1;
            }
        }
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
