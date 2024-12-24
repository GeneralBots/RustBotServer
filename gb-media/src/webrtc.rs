use gb_core::{Result, Error};
use webrtc::{
    api::{API, APIBuilder},
    peer_connection::{
        RTCPeerConnection,
        peer_connection_state::RTCPeerConnectionState,
        configuration::RTCConfiguration,
    },
    track::{
        track_local::TrackLocal,
        track_remote::TrackRemote,
    },
};
use tokio::sync::mpsc;
use tracing::instrument;
use std::sync::Arc;

pub struct WebRTCService {
    api: Arc<API>,
    peer_connections: Vec<Arc<RTCPeerConnection>>,
}

impl WebRTCService {
    pub fn new() -> Result<Self> {
        let api = APIBuilder::new().build();

        Ok(Self {
            api: Arc::new(api),
            peer_connections: Vec::new(),
        })
    }

    pub async fn create_peer_connection(&mut self) -> Result<Arc<RTCPeerConnection>> {
        let config = RTCConfiguration::default();
        
        let peer_connection = self.api.new_peer_connection(config)
            .await
            .map_err(|e| Error::internal(format!("Failed to create peer connection: {}", e)))?;

        let pc_arc = Arc::new(peer_connection);
        self.peer_connections.push(pc_arc.clone());

        Ok(pc_arc)
    }

    pub async fn add_track(
        &self,
        pc: &RTCPeerConnection,
        track: Arc<dyn TrackLocal + Send + Sync>,
    ) -> Result<()> {
        pc.add_track(track)
            .await
            .map_err(|e| Error::internal(format!("Failed to add track: {}", e)))?;

        Ok(())
    }

    pub async fn on_track<F>(&self, pc: &RTCPeerConnection, mut callback: F)
    where
        F: FnMut(Arc<TrackRemote>) + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);

        pc.on_track(Box::new(move |track, _, _| {
            let track_clone = track.clone();
            let tx = tx.clone();
            Box::pin(async move {
                let _ = tx.send(track_clone).await;
            })
        }));

        while let Some(track) = rx.recv().await {
            callback(track);
        }
    }

    #[instrument(skip(self))]
    pub async fn close(&mut self) -> Result<()> {
        for pc in self.peer_connections.iter() {
            pc.close().await
                .map_err(|e| Error::internal(format!("Failed to close peer connection: {}", e)))?;
        }
        self.peer_connections.clear();
        Ok(())
    }
}