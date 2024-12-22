use sqlx::PgPool;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations");
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS customers (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            subscription_tier VARCHAR(50) NOT NULL,
            status VARCHAR(50) NOT NULL,
            max_instances INTEGER NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS instances (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL REFERENCES customers(id),
            name VARCHAR(255) NOT NULL,
            status VARCHAR(50) NOT NULL,
            shard_id INTEGER NOT NULL,
            region VARCHAR(50) NOT NULL,
            config JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS rooms (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL REFERENCES customers(id),
            instance_id UUID NOT NULL REFERENCES instances(id),
            name VARCHAR(255) NOT NULL,
            kind VARCHAR(50) NOT NULL,
            status VARCHAR(50) NOT NULL,
            config JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS messages (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL REFERENCES customers(id),
            instance_id UUID NOT NULL REFERENCES instances(id),
            conversation_id UUID NOT NULL,
            sender_id UUID NOT NULL,
            kind VARCHAR(50) NOT NULL,
            content TEXT NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
            shard_key INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL REFERENCES customers(id),
            instance_id UUID NOT NULL REFERENCES instances(id),
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            status VARCHAR(50) NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS tracks (
            id UUID PRIMARY KEY,
            room_id UUID NOT NULL REFERENCES rooms(id),
            user_id UUID NOT NULL REFERENCES users(id),
            kind VARCHAR(50) NOT NULL,
            status VARCHAR(50) NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS subscriptions (
            id UUID PRIMARY KEY,
            track_id UUID NOT NULL REFERENCES tracks(id),
            user_id UUID NOT NULL REFERENCES users(id),
            status VARCHAR(50) NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        -- Create indexes for performance
        CREATE INDEX IF NOT EXISTS idx_instances_customer_id ON instances(customer_id);
        CREATE INDEX IF NOT EXISTS idx_rooms_instance_id ON rooms(instance_id);
        CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_messages_shard_key ON messages(shard_key);
        CREATE INDEX IF NOT EXISTS idx_tracks_room_id ON tracks(room_id);
        CREATE INDEX IF NOT EXISTS idx_subscriptions_track_id ON subscriptions(track_id);
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        "#,
    )
    .execute(pool)
    .await?;

    info!("Migrations completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::{PgPoolOptions, PgPool};
    use rstest::*;

    async fn create_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/gb_test".to_string());
            
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[rstest]
    #[tokio::test]
    async fn test_migrations() {
        let pool = create_test_pool().await;
        assert!(run_migrations(&pool).await.is_ok());
    }
}
