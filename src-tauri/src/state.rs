use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use tauri::Manager;
use tauri::{AppHandle, State};

use crate::db::ConnectionManagerSafe;
use crate::projects::{Project, ProjectId};

// State for application
pub struct AppState {
    /// Maps window IDs to projects. When we need to persist projects later,
    /// we can easily serialize them from this map.
    pub windows: RwLock<HashMap<String, Project>>,
}
pub type AppStateSafe = State<'static, AppState>;

impl Default for AppState {
    fn default() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
        }
    }
}

impl AppState {
    /// Gets an existing project by ID or creates a new one if it doesn't exist
    pub fn get_or_create_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<(Project, String), String> {
        let mut windows = self.windows.write().unwrap();
        let id_str = project_id.to_string();

        // First try to find an existing window for this project
        if let Some((window_id, project)) = windows.iter().find(|(_, p)| p.id.to_string() == id_str)
        {
            return Ok((project.clone(), window_id.clone()));
        }

        // No existing window found, create a new project
        let connection_info = crate::projects::load_connection_info(project_id)?;
        let name = project_id.derive_name();
        let project = Project::new(project_id.clone(), name, connection_info);

        // Generate new window ID and store project
        let window_id = format!("window_{}", Uuid::new_v4());
        windows.insert(window_id.clone(), project.clone());

        Ok((project, window_id))
    }

    /// Gets a project by window ID
    pub fn get_project_by_window(&self, window_id: &str) -> Option<Project> {
        let windows = self.windows.read().unwrap();
        windows.get(window_id).cloned()
    }

    /// Gets all projects
    pub fn get_all_projects(&self) -> Vec<Project> {
        let windows = self.windows.read().unwrap();
        windows.values().cloned().collect()
    }

    /// Cleanup resources for a specific window
    pub async fn cleanup_window(&self, app: &AppHandle, window_id: &str) -> Result<(), String> {
        // Get the project for this window
        let mut windows = self.windows.write().unwrap();
        if let Some(_) = windows.remove(window_id) {
            // Clean up the database connection
            let db_manager = app.state::<ConnectionManagerSafe>();
            if let Err(e) = db_manager.remove_connection(window_id).await {
                log::warn!(
                    "Failed to remove connection for window {}: {}",
                    window_id,
                    e
                );
            }
            Ok(())
        } else {
            Err(format!("No project found for window {}", window_id))
        }
    }

    /// Cleanup all resources when application is closing
    pub async fn cleanup_all(&self, app: &AppHandle) {
        let mut windows = self.windows.write().unwrap();
        let window_ids: Vec<String> = windows.keys().cloned().collect();

        // Clean up each window's resources
        for window_id in window_ids {
            if let Err(e) = self.cleanup_window(app, &window_id).await {
                log::warn!("Error cleaning up window {}: {}", window_id, e);
            }
        }

        // Clear the windows map
        windows.clear();

        // Close all database connections
        let db_manager = app.state::<ConnectionManagerSafe>();
        db_manager.close_all_connections().await;
    }

    /// Save projects to disk (to be implemented later)
    pub fn save_projects(&self) -> Result<(), String> {
        // NOTE: This will be implemented later to persist projects to the config directory
        // For now just return Ok
        Ok(())
    }

    /// Load projects from disk (to be implemented later)
    pub fn load_projects(&self) -> Result<(), String> {
        // NOTE: This will be implemented later to load projects from the config directory
        // For now just return Ok
        Ok(())
    }
}
