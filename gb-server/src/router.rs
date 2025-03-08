use axum::{
    routing::{get, post},
    Router,
    extract::{
        ws::WebSocket,
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use gb_core::{Result, Error, models::*};
use gb_messaging::{MessageProcessor, models::MessageEnvelope};
use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use tracing::{instrument, error};
use uuid::Uuid;
use futures_util::StreamExt;


pub fn create_router(message_processor: AppState) -> Router {
    let state = Arc::new(ApiState {
        message_processor: Mutex::new(message_processor),
    });
   Router::new()



   // File & Document Management
            .route("/files/upload", post(upload_file))
            .route("/files/download", post(download))
            .route("/files/copy", post(copy_file))
            .route("/files/move", post(move_file))
            .route("/files/delete", post(delete_file))
            .route("/files/getContents", post(get_file_contents))
            .route("/files/save", post(save_file))
            .route("/files/createFolder", post(create_folder))
            .route("/files/shareFolder", post(share_folder))
            .route("/files/dirFolder", post(dir_folder))
            .route("/files/list", post(get_files))
            .route("/files/search", post(search_files))
            .route("/files/recent", post(get_recent_files))
            .route("/files/favorite", post(toggle_favorite))
            .route("/files/versions", post(get_file_versions))
            .route("/files/restore", post(restore_file_version))
            .route("/files/permissions", post(set_file_permissions))
            .route("/files/quota", get(get_storage_quota))
            .route("/files/shared", get(get_shared_files))
            .route("/files/sync/status", get(get_sync_status))
            .route("/files/sync/start", post(start_sync))
            .route("/files/sync/stop", post(stop_sync))
    
            // full ode bucket is abstrctd path variable, src, dest, full file manager acessible via actixweb ALL methods no excluses, inline funcition params, s3 api inside, all methodos, full code.            // Document Processing
            
            .route("/docs/merge", post(merge_documents))
            .route("/docs/convert", post(convert_document))
            .route("/docs/fill", post(fill_document))
            .route("/docs/export", post(export_document))
            .route("/docs/import", post(import_document))
    
            // Groups & Organizations
            .route("/groups/create", post(create_group))
            .route("/groups/update", put(update_group))
            .route("/groups/delete", delete(delete_group))
            .route("/groups/list", get(get_groups))
            .route("/groups/search", post(search_groups))
            .route("/groups/members", get(get_group_members))
            .route("/groups/members/add", post(add_group_member))
            .route("/groups/members/remove", post(remove_group_member))
            .route("/groups/permissions", post(set_group_permissions))
            .route("/groups/settings", post(update_group_settings))
            .route("/groups/analytics", get(get_group_analytics))
            .route("/groups/join/request", post(request_group_join))
            .route("/groups/join/approve", post(approve_join_request))
            .route("/groups/join/reject", post(reject_join_request))
            .route("/groups/invites/send", post(send_group_invite))
            .route("/groups/invites/list", get(list_group_invites))
    
            // Conversations & Real-time Communication
            .route("/conversations/create", post(create_conversation))
            .route("/conversations/join", post(join_conversation))
            .route("/conversations/leave", post(leave_conversation))
            .route("/conversations/members", get(get_conversation_members))
            .route("/conversations/messages", get(get_messages))
            .route("/conversations/messages/send", post(send_message))
            .route("/conversations/messages/edit", put(edit_message))
            .route("/conversations/messages/delete", delete(delete_message))
            .route("/conversations/messages/react", post(react_to_message))
            .route("/conversations/messages/pin", post(pin_message))
            .route("/conversations/messages/search", post(search_messages))
            .route("/conversations/calls/start", post(start_call))
            .route("/conversations/calls/join", post(join_call))
            .route("/conversations/calls/leave", post(leave_call))
            .route("/conversations/calls/mute", post(mute_participant))
            .route("/conversations/calls/unmute", post(unmute_participant))
            .route("/conversations/screen/share", post(share_screen))
            .route("/conversations/screen/stop", post(stop_screen_share))
            .route("/conversations/recording/start", post(start_recording))
            .route("/conversations/recording/stop", post(stop_recording))
            .route("/conversations/whiteboard/create", post(create_whiteboard))
            .route("/conversations/whiteboard/collaborate", post(collaborate_whiteboard))
    
            // Communication Services
            .route("/comm/email/send", post(send_email))
            .route("/comm/email/template", post(send_template_email))
            .route("/comm/email/schedule", post(schedule_email))
            .route("/comm/email/cancel", post(cancel_scheduled_email))
            .route("/comm/sms/send", post(send_sms))
            .route("/comm/sms/bulk", post(send_bulk_sms))
            .route("/comm/notifications/send", post(send_notification))
            .route("/comm/notifications/preferences", post(set_notification_preferences))
            .route("/comm/broadcast/send", post(send_broadcast))
            .route("/comm/contacts/import", post(import_contacts))
            .route("/comm/contacts/export", post(export_contacts))
            .route("/comm/contacts/sync", post(sync_contacts))
            .route("/comm/contacts/groups", post(manage_contact_groups))
    
            // User Management & Authentication
            .route("/users/create", post(create_user))
            .route("/users/update", put(update_user))
            .route("/users/delete", delete(delete_user))
            .route("/users/list", get(get_users))
            .route("/users/search", post(search_users))
            .route("/users/profile", get(get_user_profile))
            .route("/users/profile/update", put(update_profile))
            .route("/users/settings", post(update_user_settings))
            .route("/users/permissions", post(set_user_permissions))
            .route("/users/roles", post(manage_user_roles))
            .route("/users/status", post(update_user_status))
            .route("/users/presence", get(get_user_presence))
            .route("/users/activity", get(get_user_activity))
            .route("/users/security/2fa/enable", post(enable_2fa))
            .route("/users/security/2fa/disable", post(disable_2fa))
            .route("/users/security/devices", get(get_registered_devices))
            .route("/users/security/sessions", get(get_active_sessions))
            .route("/users/notifications/settings", post(update_notification_settings))
    
            // Calendar & Task Management
            .route("/calendar/events/create", post(create_event))
            .route("/calendar/events/update", put(update_event))
            .route("/calendar/events/delete", delete(delete_event))
            .route("/calendar/events/list", get(get_calendar_events))
            .route("/calendar/events/search", post(search_events))
            .route("/calendar/availability/check", post(check_availability))
            .route("/calendar/schedule/meeting", post(schedule_meeting))
            .route("/calendar/reminders/set", post(set_reminder))
            .route("/tasks/create", post(create_task))
            .route("/tasks/update", put(update_task))
            .route("/tasks/delete", delete(delete_task))
            .route("/tasks/list", get(get_tasks))
            .route("/tasks/assign", post(assign_task))
            .route("/tasks/status/update", put(update_task_status))
            .route("/tasks/priority/set", post(set_task_priority))
            .route("/tasks/dependencies/set", post(set_task_dependencies))
    
            // Storage & Data Management
            .route("/storage/save", post(save_to_storage))
            .route("/storage/batch", post(save_batch_to_storage))
            .route("/storage/json", post(save_json_to_storage))
            .route("/storage/delete", delete(delete_from_storage))
            .route("/storage/quota/check", get(check_storage_quota))
            .route("/storage/cleanup", post(cleanup_storage))
            .route("/storage/backup/create", post(create_backup))
            .route("/storage/backup/restore", post(restore_backup))
            .route("/storage/archive", post(archive_data))
            .route("/storage/metrics", get(get_storage_metrics))
    
    
            // Analytics & Reporting
            .route("/analytics/dashboard", get(get_dashboard_data))
            .route("/analytics/reports/generate", post(generate_report))
            .route("/analytics/reports/schedule", post(schedule_report))
            .route("/analytics/metrics/collect", post(collect_metrics))
            .route("/analytics/insights/generate", post(generate_insights))
            .route("/analytics/trends/analyze", post(analyze_trends))
            .route("/analytics/export", post(export_analytics))
    
            // System & Administration
            .route("/admin/system/status", get(get_system_status))
            .route("/admin/system/metrics", get(get_system_metrics))
            .route("/admin/logs/view", get(view_logs))
            .route("/admin/logs/export", post(export_logs))
            .route("/admin/config/update", post(update_config))
            .route("/admin/maintenance/schedule", post(schedule_maintenance))
            .route("/admin/backup/create", post(create_system_backup))
            .route("/admin/backup/restore", post(restore_system_backup))
            .route("/admin/users/manage", post(manage_system_users))
            .route("/admin/roles/manage", post(manage_system_roles))
            .route("/admin/quotas/manage", post(manage_quotas))
            .route("/admin/licenses/manage", post(manage_licenses))
    
            // AI & Machine Learning
            .route("/ai/analyze/text", post(analyze_text))
            .route("/ai/analyze/image", post(analyze_image))
            .route("/ai/generate/text", post(generate_text))
            .route("/ai/generate/image", post(generate_image))
            .route("/ai/translate", post(translate_content))
            .route("/ai/summarize", post(summarize_content))
            .route("/ai/recommend", post(get_recommendations))
            .route("/ai/train/model", post(train_custom_model))
            .route("/ai/predict", post(make_prediction))
    
            // Security & Compliance
            .route("/security/audit/logs", get(get_audit_logs))
            .route("/security/compliance/check", post(check_compliance))
            .route("/security/threats/scan", post(scan_for_threats))
            .route("/security/access/review", post(review_access))
            .route("/security/encryption/manage", post(manage_encryption))
            .route("/security/certificates/manage", post(manage_certificates))
    
            // Health & Monitoring
            .route("/health", get(health_check))
            .route("/health/detailed", get(detailed_health_check))
            .route("/monitoring/status", get(get_monitoring_status))
            .route("/monitoring/alerts", get(get_active_alerts))
            .route("/monitoring/metrics", get(get_monitoring_metrics))
             .with_state(state)
}

async fn handle_ws_connection(
    ws: WebSocket,
    state: Arc<ApiState>,
) -> Result<()> {
    let (_sender, mut receiver) = ws.split();
    
    while let Some(Ok(msg)) = receiver.next().await {
        if let Ok(text) = msg.to_text() {
            if let Ok(_envelope) = serde_json::from_str::<MessageEnvelope>(text) {
                let mut processor = state.message_processor.lock().await;
                if let Err(e) = processor.process_messages().await {
                    error!("Failed to process message: {}", e);
                }
            }
        }
    }
    Ok(())
}

#[instrument(skip_all)]
async fn websocket_handler(
    State(state): State<Arc<ApiState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let _ = handle_ws_connection(socket, state).await;
    })
}

#[instrument(skip_all)]
async fn send_message(
    State(state): State<Arc<ApiState>>,
    Json(message): Json<Message>,
) -> Result<Json<MessageId>> {
    // Clone the message before using it in envelope
    let envelope = MessageEnvelope {
        id: Uuid::new_v4(),
        message: message.clone(),  // Clone here
        metadata: HashMap::new(),
    };

    let mut processor = state.message_processor.lock().await;
    processor.add_message(message)  // Use original message here
        .await
        .map_err(|e| Error::internal(format!("Failed to add message: {}", e)))?;
    
    Ok(Json(MessageId(envelope.id)))
}

#[instrument(skip_all)]
async fn get_message(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<Message>> {
    todo!()
}

#[instrument(skip_all)]
async fn create_room(
    State(_state): State<Arc<ApiState>>,
    Json(_config): Json<RoomConfig>,
) -> Result<Json<Room>> {
    todo!()
}

#[instrument(skip_all)]
async fn get_room(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<Room>> {
    todo!()
}

#[instrument(skip_all)]
async fn join_room(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<Uuid>,
    Json(_user_id): Json<Uuid>,
) -> Result<Json<Connection>> {
    todo!()
}