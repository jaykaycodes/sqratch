use tauri::{Runtime, Window};
use taurpc;

use crate::errors::AppError;
use crate::project::Project;
use crate::state::get_window_project;

#[taurpc::procedures(path = "projects", export_to = "../src/lib/taurpc.ts", event_trigger = ProjectEventTrigger)]
pub trait ProjectsApi {
    async fn get_project(window: Window<impl Runtime>) -> Result<Project, AppError>;
}

#[derive(Clone)]
pub struct ProjectsApiImpl;

#[taurpc::resolvers]
impl ProjectsApi for ProjectsApiImpl {
    async fn get_project(self, window: Window<impl Runtime>) -> Result<Project, AppError> {
        let project = get_window_project(&window)?;
        Ok((*project).clone())
    }
}
