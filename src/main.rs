use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

use services::script
::*;
use services::config::*;
use services::email::*;
use services::file::*;
use services::state::*;
use services::llm::*;

mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = AppConfig::from_env();

   let script_service = ScriptService::new();
    
    let script = r#"
    let json = FIND "users", "name=John"
    let x=2
    let text = GET "example.com"
    let nome = "table"
    let d = FIND nome, "car=2"
    "#;
    
    match script_service.compile(script) {
        Ok(ast) => {
            match script_service.run(&ast) {
                Ok(result) => println!("Script executed successfully: {:?}", result),
                Err(e) => eprintln!("Error executing script: {}", e),
            }
        },
        Err(e) => eprintln!("Error compiling script: {}", e),
    }


    let db_url = config.database_url();
    //let db = PgPool::connect(&db_url).await.unwrap();

    // let minio_client = init_minio(&config)
    //     .await
    //     .expect("Failed to initialize Minio");

    let app_state = web::Data::new(AppState {
        db: None,
        config: Some(config.clone()),
        minio_client: None,
    });
    
    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Your Next.js port
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(upload_file)
            .service(list_file)
            .service(save_click)
            .service(get_emails)
            .service(list_emails)
            .service(send_email)
            .service(chat_stream)
            .service(chat)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
