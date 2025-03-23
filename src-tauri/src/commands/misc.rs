use crate::project;

/// Simple greeting command for testing
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Gets project configuration
#[tauri::command]
pub async fn get_project_config(
    project_path: String,
) -> Result<serde_json::Value, String> {
    match project::ProjectManager::parse_config(&project_path) {
        Ok(config) => {
            // Convert to JSON and return
            serde_json::to_value(config)
                .map_err(|e| format!("Failed to serialize project config: {}", e))
        }
        Err(e) => Err(format!("Failed to parse project config: {}", e)),
    }
}
