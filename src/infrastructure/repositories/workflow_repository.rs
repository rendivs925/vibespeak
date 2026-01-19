use crate::domain::entities::Workflow;
use crate::shared::{Error, Result, WorkflowId};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Workflow repository trait for workflow persistence
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &Workflow) -> Result<()>;
    async fn find_by_id(&self, id: &WorkflowId) -> Result<Option<Workflow>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Workflow>>;
    async fn find_all(&self) -> Result<Vec<Workflow>>;
    async fn find_enabled(&self) -> Result<Vec<Workflow>>;
    async fn delete(&self, id: &WorkflowId) -> Result<()>;
    async fn count(&self) -> Result<usize>;
}

/// In-memory workflow repository
pub struct InMemoryWorkflowRepository {
    workflows: RwLock<HashMap<WorkflowId, Workflow>>,
}

impl InMemoryWorkflowRepository {
    pub fn new() -> Self {
        Self {
            workflows: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl WorkflowRepository for InMemoryWorkflowRepository {
    async fn save(&self, workflow: &Workflow) -> Result<()> {
        let mut workflows = self.workflows.write().unwrap();
        workflows.insert(workflow.id.clone(), workflow.clone());
        tracing::info!("Saved workflow: {}", workflow.name);
        Ok(())
    }

    async fn find_by_id(&self, id: &WorkflowId) -> Result<Option<Workflow>> {
        let workflows = self.workflows.read().unwrap();
        Ok(workflows.get(id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Workflow>> {
        let workflows = self.workflows.read().unwrap();
        Ok(workflows.values().find(|w| w.name == name).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Workflow>> {
        let workflows = self.workflows.read().unwrap();
        Ok(workflows.values().cloned().collect())
    }

    async fn find_enabled(&self) -> Result<Vec<Workflow>> {
        let workflows = self.workflows.read().unwrap();
        Ok(workflows.values().filter(|w| w.enabled).cloned().collect())
    }

    async fn delete(&self, id: &WorkflowId) -> Result<()> {
        let mut workflows = self.workflows.write().unwrap();
        if workflows.remove(id).is_some() {
            tracing::info!("Deleted workflow: {}", id);
            Ok(())
        } else {
            Err(Error::Infrastructure(format!("Workflow {} not found", id)))
        }
    }

    async fn count(&self) -> Result<usize> {
        let workflows = self.workflows.read().unwrap();
        Ok(workflows.len())
    }
}

/// JSON file-based persistent workflow repository
pub struct JsonFileWorkflowRepository {
    file_path: PathBuf,
    cache: RwLock<HashMap<WorkflowId, Workflow>>,
}

impl JsonFileWorkflowRepository {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let repo = Self {
            file_path,
            cache: RwLock::new(HashMap::new()),
        };
        repo.load_from_file()?;
        Ok(repo)
    }

    /// Create with default data directory
    pub fn with_default_path() -> Result<Self> {
        let path = PathBuf::from("data/workflows.json");
        Self::new(path)
    }

    fn load_from_file(&self) -> Result<()> {
        if self.file_path.exists() {
            let content = std::fs::read_to_string(&self.file_path).map_err(|e| {
                Error::Infrastructure(format!("Failed to read workflows file: {}", e))
            })?;
            let workflows: Vec<Workflow> = serde_json::from_str(&content).map_err(|e| {
                Error::Infrastructure(format!("Failed to parse workflows file: {}", e))
            })?;

            let mut cache = self.cache.write().unwrap();
            for workflow in workflows {
                cache.insert(workflow.id.clone(), workflow);
            }
            tracing::info!(
                "Loaded {} workflows from {}",
                cache.len(),
                self.file_path.display()
            );
        } else {
            tracing::info!("Workflows file does not exist, starting with empty repository");
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Infrastructure(format!("Failed to create data directory: {}", e))
            })?;
        }

        let workflows = self.cache.read().unwrap();
        let workflows_vec: Vec<&Workflow> = workflows.values().collect();
        let content = serde_json::to_string_pretty(&workflows_vec)
            .map_err(|e| Error::Infrastructure(format!("Failed to serialize workflows: {}", e)))?;

        std::fs::write(&self.file_path, content)
            .map_err(|e| Error::Infrastructure(format!("Failed to write workflows file: {}", e)))?;

        tracing::debug!(
            "Saved {} workflows to {}",
            workflows.len(),
            self.file_path.display()
        );
        Ok(())
    }
}

#[async_trait]
impl WorkflowRepository for JsonFileWorkflowRepository {
    async fn save(&self, workflow: &Workflow) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(workflow.id.clone(), workflow.clone());
        }
        self.save_to_file()?;
        tracing::info!("Saved workflow: {} (persistent)", workflow.name);
        Ok(())
    }

    async fn find_by_id(&self, id: &WorkflowId) -> Result<Option<Workflow>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.get(id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Workflow>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.values().find(|w| w.name == name).cloned())
    }

    async fn find_all(&self) -> Result<Vec<Workflow>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.values().cloned().collect())
    }

    async fn find_enabled(&self) -> Result<Vec<Workflow>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.values().filter(|w| w.enabled).cloned().collect())
    }

    async fn delete(&self, id: &WorkflowId) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            if cache.remove(id).is_none() {
                return Err(Error::Infrastructure(format!("Workflow {} not found", id)));
            }
        }
        self.save_to_file()?;
        tracing::info!("Deleted workflow: {} (persistent)", id);
        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        let cache = self.cache.read().unwrap();
        Ok(cache.len())
    }
}
