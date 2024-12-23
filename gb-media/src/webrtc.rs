use async_trait::async_trait;
use gb_core::{
    models::*,
    traits::*,
    Result, Error, Connection,
};
use uuid::Uuid;
use webrtc::{
    api::APIBuilder,
    ice_transport::ice_server::RTCIceServer,
    peer_connection::configuration::RTCConfiguration,
    peer_connection::peer_connection_state::RTCPeerConnectionState,
    peer_connection::RTCPeerConnection,
    track::track_remote::TrackRemote,
    rtp::rtp_receiver::RTCRtpReceiver,
    rtp::rtp_transceiver::RTCRtpTransceiver,
};
use tracing::{instrument, error};
use std::sync::Arc;
use chrono::Utc;

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
            .map_err(|e| Error::internal(format!("Failed to create peer connection: {}", e)))?;

        Ok(peer_connection)
    }

    async fn handle_track(&self, track: Arc<TrackRemote>, receiver: Arc<RTCRtpReceiver>, transceiver: Arc<RTCRtpTransceiver>) {
                        tracing::info!(
            "Received track: {} {}", 
                            track.kind(),
                            track.id()
                        );
                    }

    async fn create_connection(&self) -> Result<Connection> {
        Ok(Connection {
            id: Uuid::new_v4(),
            connected_at: Utc::now(),
            ice_servers: self.config.ice_servers.clone(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            room_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
        })
    }
}

#[async_trait]
impl RoomService for WebRTCService {
    #[instrument(skip(self))]
    async fn create_room(&self, config: RoomConfig) -> Result<Room> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<Connection> {
        let peer_connection = self.create_peer_connection().await?;

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

        let mut connection = self.create_connection().await?;
        connection.room_id = room_id;
        connection.user_id = user_id;

        Ok(connection)
    }

    #[instrument(skip(self))]
    async fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn publish_track(&self, track: TrackInfo) -> Result<Track> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn subscribe_track(&self, track_id: Uuid) -> Result<Subscription> {
        todo!()
}

    #[instrument(skip(self))]
    async fn get_participants(&self, room_id: Uuid) -> Result<Vec<Participant>> {
        todo!()
    }

    #[instrument(skip(self))]
    async fn get_room_stats(&self, room_id: Uuid) -> Result<RoomStats> {
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
