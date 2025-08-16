#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bot {
    pub bot_id: Uuid,
    pub name: String,
    pub status: BotStatus,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "bot_status", rename_all = "snake_case")]
pub enum BotStatus {
    Active,
    Inactive,
    Maintenance,
}
