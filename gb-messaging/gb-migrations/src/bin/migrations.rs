use sqlx::PgPool;
use gb_migrations::run_migrations;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    println!("Creating database connection pool...");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    println!("Running migrations...");
    run_migrations(&pool).await?;
    
    println!("Migrations completed successfully!");
    Ok(())
}