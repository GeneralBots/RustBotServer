use rand::{distributions::Alphanumeric, Rng};
use std::time::Duration;

/// Generates a random alphanumeric string of the specified length
///
/// # Arguments
/// * `length` - The desired length of the random string
///
/// # Returns
/// A String containing random alphanumeric characters
#[must_use]
pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generates a vector of random bytes for testing purposes
///
/// # Arguments
/// * `size` - The number of random bytes to generate
///
/// # Returns
/// A Vec<u8> containing random bytes
#[must_use]
pub fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen::<u8>()).collect()
}

/// Executes an operation with exponential backoff retry strategy
///
/// # Arguments
/// * `operation` - The async operation to execute
/// * `max_retries` - Maximum number of retry attempts
/// * `initial_delay` - Initial delay duration between retries
///
/// # Returns
/// Result containing the operation output or an error
///
/// # Errors
/// Returns the last error encountered after all retries are exhausted
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
                    return Err(anyhow::anyhow!("Operation failed after {} retries: {}", max_retries, error));
                }
                tokio::time::sleep(delay).await;
                delay = delay.saturating_mul(2); // Prevent overflow
                retries += 1;
            }
        }
    }
}

/// Formats a Duration into a human-readable string in HH:MM:SS format
///
/// # Arguments
/// * `duration` - The Duration to format
///
/// # Returns
/// A String in the format "HH:MM:SS"
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let length = 10;
        let result = generate_random_string(length);
        assert_eq!(result.len(), length);
    }

    #[test]
    fn test_generate_test_data() {
        let size = 100;
        let result = generate_test_data(size);
        assert_eq!(result.len(), size);
    }

    #[test]
    fn test_format_duration() {
        let duration = Duration::from_secs(3661); // 1 hour, 1 minute, 1 second
        assert_eq!(format_duration(duration), "01:01:01");
    }

    
    #[tokio::test]
    async fn test_exponential_backoff() {
        // Use interior mutability with RefCell to allow mutation in the closure
        use std::cell::RefCell;
        let counter = RefCell::new(0);
        
        let operation = || async {
            *counter.borrow_mut() += 1;
            if *counter.borrow() < 3 {
                Err(anyhow::anyhow!("Test error"))
            } else {
                Ok(*counter.borrow())
            }
        };

        let result = exponential_backoff(
            operation,
            5,
            Duration::from_millis(1),
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }
}