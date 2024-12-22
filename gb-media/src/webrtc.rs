use async_trait::async_trait;
use gb_core::{
    models::*,
    traits::*,
    Result, Error,
};
use uuid::Uuid;
use webrtc::{
    api::APIBuilder,
    ice_transport::ice_server::RTCIceServer,
    peer_connection::configuration::RTCConfiguration,
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    peer_connection::RTCPeerConnection,
    track::track_remote::TrackRemote,
};
use tracing::{instrument, error};

pub struct WebRTCService {
    config: RTCConfiguration,
}

impl WebRTCService {
    pub fn new(ice_servers: Vec<String>) -> Self {
        let mut config = RTCConfiguration::default();
        config.ice_servers = ice_servers
            .into_iter()
            .map(|url| RTCIceServer {
                urls: vec![url],
                ..Default::default()
            })
            .collect();

        Self { config }
    }

    async fn create_peer_connection(&self) -> Result<RTCPeerConnection> {
        let api = APIBuilder::new().build();
        
        let peer_connection = api.new_peer_connection(self.config.clone())
            .await
            .map_err(|e| Error::WebRTC(format!("Failed to create peer connection: {}", e)))?;

        Ok(peer_connection)
    }
}

#[async_trait]
impl RoomService for WebRTCService {
    #[instrument(skip(self))]
    async fn create_room(&self, config: RoomConfig) -> Result<Room> {
        // Create room implementation
        todo!()
    }

    #[instrument(skip(self))]
    async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<Connection> {
        let peer_connection = self.create_peer_connection().await?;

        // Setup connection handlers
        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                Box::pin(async move {
                    match s {
                        RTCPeerConnectionState::Connected => {
                            tracing::info!("Peer connection connected");
                        }
                        RTCPeerConnectionState::Disconnected
                        | RTCPeerConnectionState::Failed
                        | RTCPeerConnectionState::Closed => {
                            tracing::warn!("Peer connection state changed to {}", s);
                        }
                        _ => {}
                    }
                })
            }));

        peer_connection
            .on_track(Box::new(move |track: Option<Arc<TrackRemote>>, _receiver| {
                Box::pin(async move {
                    if let Some(track) = track {
                        tracing::info!(
                            "Received track: {} {}", 
                            track.kind(),
                            track.id()
                        );
                    }
                })
            }));

        // Create connection object
        let connection = Connection {
            id: Uuid::new_v4(),
            room_id,
            user_id,
            ice_servers: self.config.ice_servers.clone(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        Ok(connection)
    }

    #[instrument(skip(self))]
    async fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()> {
        // Leave room implementation 
        todo!()
    }

    #[instrument(skip(self))]
    async fn publish_track(&self, track: TrackInfo) -> Result<Track> {
        // Publish track implementation
        todo!()
    }

    #[instrument(skip(self))]
    async fn subscribe_track(&self, track_id: Uuid) -> Result<Subscription> {
        // Subscribe to track implementation
        todo!()
    }

    #[instrument(skip(self))]
    async fn get_participants(&self, room_id: Uuid) -> Result<Vec<Participant>> {
        // Get participants implementation
        todo!()
    }

    #[instrument(skip(self))]
    async fn get_room_stats(&self, room_id: Uuid) -> Result<RoomStats> {
        // Get room stats implementation
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn webrtc_service() -> WebRTCService {
        WebRTCService::new(vec!["stun:stun.l.google.com:19302".to_string()])
    }

    #[rstest]
    #[tokio::test]

    async fn test_create_peer_connection(webrtc_service: WebRTCService) {
        let peer_connection = webrtc_service.create_peer_connection().await.unwrap();
        assert_eq!(
            peer_connection.connection_state().await,
            RTCPeerConnectionState::New
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_join_room(webrtc_service: WebRTCService) {
        let room_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        let connection = webrtc_service.join_room(room_id, user_id).await.unwrap();
        
        assert_eq!(connection.room_id, room_id);
        assert_eq!(connection.user_id, user_id);
        assert!(!connection.ice_servers.is_empty());
    }
}
