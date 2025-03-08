use actix_web::web;

use crate::router;

pub fn files_router_configure(cfg: &mut web::ServiceConfig) {
    // File & Document Management
    cfg.route("/files/upload", web::post().to(handlers::upload_file))
        .route("/files/download", web::post().to(handlers::download))
        .route("/files/delete", web::post().to(handlers::delete_file))
        .route("/files/getContents", web::post().to(handlers::get_file_contents))
        .route("/files/createFolder", web::post().to(handlers::create_folder))
        .route("/files/dirFolder", web::post().to(handlers::dir_folder))
        
        // Conversations & Real-time Communication
        .route("/conversations/create", web::post().to(handlers::create_conversation))
        .route("/conversations/join", web::post().to(handlers::join_conversation))
        .route("/conversations/leave", web::post().to(handlers::leave_conversation))
        .route("/conversations/members", web::get().to(handlers::get_conversation_members))
        .route("/conversations/messages", web::get().to(handlers::get_messages))
        .route("/conversations/messages/send", web::post().to(handlers::send_message))
        
        // Communication Services
        .route("/comm/email/send", web::post().to(handlers::send_email))
        
        // User Management
        .route("/users/profile", web::get().to(handlers::get_user_profile))
        
        // Calendar & Task Management
        .route("/calendar/events/create", web::post().to(handlers::create_event))
        
        .route("/tasks/create", web::post().to(handlers::create_task))
        .route("/tasks/list", web::get().to(handlers::get_tasks))
        
        // Admin
        .route("/admin/system/status", web::get().to(handlers::get_system_status))
        .route("/admin/logs/view", web::get().to(handlers::view_logs));
}
