pub mod webrtc;
pub mod processor;
pub mod audio;

pub use webrtc::WebRTCService;
pub use processor::{MediaProcessor, MediaMetadata};
pub use audio::AudioProcessor;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_media_integration() {
        // Initialize services
        let webrtc = WebRTCService::new(vec!["stun:stun.l.google.com:19302".to_string()]);
        let processor = MediaProcessor::new().unwrap();
        let audio = AudioProcessor::new(48000, 2);

        // Test room creation and joining
        let room_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let connection = webrtc.join_room(room_id, user_id).await.unwrap();
        assert_eq!(connection.room_id, room_id);
        assert_eq!(connection.user_id, user_id);

        // Test media processing
        let input_path = PathBuf::from("test_data/test.mp4");
        if input_path.exists() {
            let metadata = processor.extract_metadata(input_path.clone()).await.unwrap();
            assert!(metadata.width.is_some());
            assert!(metadata.height.is_some());
        }

        // Test audio processing
        let test_audio: Vec<i16> = (0..1024).map(|i| i as i16).collect();
        let encoded = audio.encode(&test_audio).unwrap();
        let decoded = audio.decode(&encoded).unwrap();
        assert!(!decoded.is_empty());
    }
}
