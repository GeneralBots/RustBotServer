use crate::models::automation_model::{Automation, TriggerKind};
use crate::services::script::ScriptService;
use crate::services::state::AppState;
use chrono::Datelike;
use chrono::Timelike;
use chrono::{DateTime, Utc};
use std::path::Path;
use tokio::time::Duration;
use uuid::Uuid;

pub struct AutomationService {
    state: AppState, // Use web::Data directly
    scripts_dir: String,
}

impl AutomationService {
    pub fn new(state: AppState, scripts_dir: &str) -> Self {
        Self {
            state,
            scripts_dir: scripts_dir.to_string(),
        }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            let mut last_check = Utc::now();

            loop {
                interval.tick().await;

                if let Err(e) = self.run_cycle(&mut last_check).await {
                    eprintln!("Automation cycle error: {}", e);
                }
            }
        })
    }

    async fn run_cycle(
        &self,
        last_check: &mut DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let automations = self.load_active_automations().await?;
        self.check_table_changes(&automations, *last_check).await;
        self.process_schedules(&automations).await;
        *last_check = Utc::now();
        Ok(())
    }

    async fn load_active_automations(&self) -> Result<Vec<Automation>, sqlx::Error> {
        if let Some(pool) = &self.state.db {
            sqlx::query_as::<_, Automation>(
                r#"
                SELECT id, kind, target, schedule, param, is_active, last_triggered
                FROM system_automations
                WHERE is_active = true
                "#,
            )
            .fetch_all(pool)
            .await
        } else {
            Err(sqlx::Error::PoolClosed)
        }
    }

    async fn check_table_changes(&self, automations: &[Automation], since: DateTime<Utc>) {
        if let Some(pool) = &self.state.db_custom {
            for automation in automations {
                if let Some(trigger_kind) = TriggerKind::from_i32(automation.kind) {
                    if matches!(
                        trigger_kind,
                        TriggerKind::TableUpdate
                            | TriggerKind::TableInsert
                            | TriggerKind::TableDelete
                    ) {
                        if let Some(table) = &automation.target {
                            let column = match trigger_kind {
                                TriggerKind::TableInsert => "created_at",
                                _ => "updated_at",
                            };

                            let query =
                                format!("SELECT COUNT(*) FROM {} WHERE {} > $1", table, column);

                            match sqlx::query_scalar::<_, i64>(&query)
                                .bind(since)
                                .fetch_one(pool)
                                .await
                            {
                                Ok(count) => {
                                    if count > 0 {
                                        self.execute_action(&automation.param).await;
                                        self.update_last_triggered(automation.id).await;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error checking changes for table {}: {}", table, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process_schedules(&self, automations: &[Automation]) {
        let now = Utc::now().timestamp();

        for automation in automations {
            if let Some(TriggerKind::Scheduled) = TriggerKind::from_i32(automation.kind) {
                if let Some(pattern) = &automation.schedule {
                    if Self::should_run_cron(pattern, now) {
                        self.execute_action(&automation.param).await;
                        self.update_last_triggered(automation.id).await;
                    }
                }
            }
        }
    }

    async fn update_last_triggered(&self, automation_id: Uuid) {
        if let Some(pool) = &self.state.db {
            if let Err(e) = sqlx::query!(
                "UPDATE system_automations SET last_triggered = $1 WHERE id = $2",
                Utc::now(),
                automation_id
            )
            .execute(pool)
            .await
            {
                eprintln!(
                    "Failed to update last_triggered for automation {}: {}",
                    automation_id, e
                );
            }
        }
    }

    fn should_run_cron(pattern: &str, timestamp: i64) -> bool {
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        if parts.len() != 5 {
            return false;
        }

        let dt = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
        let minute = dt.minute() as i32;
        let hour = dt.hour() as i32;
        let day = dt.day() as i32;
        let month = dt.month() as i32;
        let weekday = dt.weekday().num_days_from_monday() as i32;

        [minute, hour, day, month, weekday]
            .iter()
            .enumerate()
            .all(|(i, &val)| Self::cron_part_matches(parts[i], val))
    }

    fn cron_part_matches(part: &str, value: i32) -> bool {
        if part == "*" {
            return true;
        }
        if part.contains('/') {
            let parts: Vec<&str> = part.split('/').collect();
            if parts.len() != 2 {
                return false;
            }
            let step: i32 = parts[1].parse().unwrap_or(1);
            if parts[0] == "*" {
                return value % step == 0;
            }
        }
        part.parse::<i32>().map_or(false, |num| num == value)
    }

    async fn execute_action(&self, param: &str) {
        let full_path = Path::new(&self.scripts_dir).join(param);
        match tokio::fs::read_to_string(&full_path).await {
            Ok(script_content) => {
                println!("Executing action with param: {}", param);

                let script_service = ScriptService::new(&self.state.clone());

                match script_service.compile(&script_content) {
                    Ok(ast) => match script_service.run(&ast) {
                        Ok(result) => println!("Script executed successfully: {:?}", result),
                        Err(e) => eprintln!("Error executing script: {}", e),
                    },
                    Err(e) => eprintln!("Error compiling script: {}", e),
                }
            }
            Err(e) => {
                eprintln!("Failed to execute action {}: {}", full_path.display(), e);
            }
        }
    }
}
