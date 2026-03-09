//! HeXuS Desktop Application
//!
//! Tauri-based desktop dashboard for biometric monitoring and analysis.

use hexus_core::{Alter, Switch};
use serde::{Deserialize, Serialize};

/// Get all alters from the system
#[tauri::command]
async fn get_alters() -> Result<Vec<Alter>, String> {
    // TODO: Query from database via hexus-core
    // For now, return stub data
    Ok(vec![
        Alter {
            id: "alter-1".to_string(),
            name: "Alice".to_string(),
            pronouns: Some("she/her".to_string()),
            color: Some("#ff6b6b".to_string()),
            description: None,
            avatar_url: None,
            created_at: chrono::Utc::now(),
        },
        Alter {
            id: "alter-2".to_string(),
            name: "Bob".to_string(),
            pronouns: Some("he/him".to_string()),
            color: Some("#4ecdc4".to_string()),
            description: None,
            avatar_url: None,
            created_at: chrono::Utc::now(),
        },
    ])
}

/// Get recent switch history
#[tauri::command]
async fn get_switches(limit: Option<usize>) -> Result<Vec<Switch>, String> {
    // TODO: Query from database via hexus-core
    // For now, return stub data
    let limit = limit.unwrap_or(10);
    
    Ok(vec![
        Switch {
            id: "switch-1".to_string(),
            timestamp: chrono::Utc::now() - chrono::Duration::hours(2),
            alter_ids: vec!["alter-1".to_string()],
            co_conscious: false,
            notes: Some("Natural switch after lunch".to_string()),
            trigger: None,
        },
        Switch {
            id: "switch-2".to_string(),
            timestamp: chrono::Utc::now() - chrono::Duration::hours(6),
            alter_ids: vec!["alter-2".to_string()],
            co_conscious: false,
            notes: Some("Morning routine switch".to_string()),
            trigger: None,
        },
    ])
}

/// Log a new switch event
#[tauri::command]
async fn log_switch(
    alter_ids: Vec<String>,
    co_conscious: bool,
    notes: Option<String>,
    trigger: Option<String>,
) -> Result<Switch, String> {
    // TODO: Store in database via hexus-core
    // For now, return stub response
    Ok(Switch {
        id: format!("switch-{}", uuid::Uuid::new_v4()),
        timestamp: chrono::Utc::now(),
        alter_ids,
        co_conscious,
        notes,
        trigger,
    })
}

/// Get system status and stats
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_alters: usize,
    pub total_switches: usize,
    pub current_fronter: Option<String>,
    pub last_switch: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
async fn get_status() -> Result<SystemStatus, String> {
    // TODO: Query from database via hexus-core
    Ok(SystemStatus {
        total_alters: 2,
        total_switches: 147,
        current_fronter: Some("Alice".to_string()),
        last_switch: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_alters,
            get_switches,
            log_switch,
            get_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
