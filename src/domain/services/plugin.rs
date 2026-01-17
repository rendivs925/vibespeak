use crate::shared::{Error, PluginId, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCapability {
    CommandProvider,
    WorkflowProvider,
    ScriptProvider,
    BrowserProvider,
    IntegrationProvider,
}

#[derive(Debug)]
pub struct PluginContext {
    pub config: HashMap<String, serde_json::Value>,
    pub shared_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct PluginInput {
    pub command: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: PluginContext,
}

#[derive(Debug)]
pub struct PluginOutput {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: Option<String>,
}

#[async_trait]
pub trait VoicePlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn initialize(&self, context: &PluginContext) -> Result<()>;
    async fn execute(&self, input: PluginInput) -> Result<PluginOutput>;
    async fn cleanup(&self) -> Result<()>;
}

pub struct PluginRegistry {
    plugins: HashMap<PluginId, Box<dyn VoicePlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn VoicePlugin>) -> Result<()> {
        let metadata = plugin.metadata();
        if self.plugins.contains_key(&metadata.id) {
            return Err(Error::Plugin(format!(
                "Plugin {} already registered",
                metadata.id
            )));
        }
        self.plugins.insert(metadata.id, plugin);
        Ok(())
    }

    pub fn get_plugin(&self, id: &PluginId) -> Option<&dyn VoicePlugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    pub fn has_capability(&self, capability: &PluginCapability) -> Vec<PluginId> {
        self.plugins
            .iter()
            .filter(|(_, plugin)| plugin.metadata().capabilities.contains(capability))
            .map(|(id, _)| id.clone())
            .collect()
    }
}
