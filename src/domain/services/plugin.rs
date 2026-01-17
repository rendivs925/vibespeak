use crate::shared::{Error, PluginId, Result};
use async_trait::async_trait;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<PluginCapability>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCapability {
    CommandProvider,
    WorkflowProvider,
    ScriptProvider,
    BrowserProvider,
    IntegrationProvider(String),
}

#[derive(Clone)]
pub struct PluginContext {
    pub config: HashMap<String, serde_json::Value>,
    pub shared_data: HashMap<String, serde_json::Value>,
    pub plugin_registry: Arc<PluginRegistry>,
}

#[derive(Clone)]
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
    async fn execute(&self, input: &PluginInput) -> Result<PluginOutput>;
    async fn cleanup(&self) -> Result<()>;
}

pub struct PluginRegistry {
    plugins: HashMap<PluginId, Box<dyn VoicePlugin>>,
    libraries: Vec<Library>, // Keep libraries loaded
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libraries: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn VoicePlugin>) -> Result<()> {
        let metadata = plugin.metadata();
        if self.plugins.contains_key(&metadata.id) {
            return Err(Error::Plugin(format!("Plugin {} already registered", metadata.id)));
        }

        tracing::info!("Registering plugin: {} v{}", metadata.name, metadata.version);
        self.plugins.insert(metadata.id, plugin);
        Ok(())
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let library = Library::new(path)
                .map_err(|e| Error::Plugin(format!("Failed to load library {}: {}", path.display(), e)))?;

            // Look for the create_plugin function
            let create_fn: Symbol<extern "C" fn() -> *mut dyn VoicePlugin> = library
                .get(b"create_plugin")
                .map_err(|e| Error::Plugin(format!("Plugin entry point not found: {}", e)))?;

            let plugin_ptr = create_fn();
            if plugin_ptr.is_null() {
                return Err(Error::Plugin("Plugin creation returned null pointer".to_string()));
            }

            // Convert to Box (this is unsafe and simplified - in practice you'd need proper memory management)
            let plugin = Box::from_raw(plugin_ptr);

            self.register(plugin)?;
            self.libraries.push(library);
        }

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

    pub async fn initialize_all(&self) -> Result<()> {
        let context = PluginContext {
            config: HashMap::new(),
            shared_data: HashMap::new(),
            plugin_registry: Arc::new(PluginRegistry::new()), // Simplified
        };

        for plugin in self.plugins.values() {
            plugin.initialize(&context).await?;
        }
        Ok(())
    }
}

// Built-in command plugin
pub struct BuiltinCommandsPlugin;

#[async_trait]
impl VoicePlugin for BuiltinCommandsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "builtin-commands".to_string(),
            name: "Built-in Commands".to_string(),
            version: "1.0.0".to_string(),
            description: "Core voice commands for system control".to_string(),
            author: "Vibespeak Team".to_string(),
            capabilities: vec![PluginCapability::CommandProvider],
            dependencies: vec![],
        }
    }

    async fn initialize(&self, _context: &PluginContext) -> Result<()> {
        tracing::info!("Built-in commands plugin initialized");
        Ok(())
    }

    async fn execute(&self, input: &PluginInput) -> Result<PluginOutput> {
        match input.command.as_str() {
            "hello" | "hi" => Ok(PluginOutput {
                success: true,
                data: serde_json::json!({"response": "Hello! How can I help you?"}),
                message: Some("Greeting acknowledged".to_string()),
            }),
            "goodbye" | "bye" => Ok(PluginOutput {
                success: true,
                data: serde_json::json!({"response": "Goodbye! Have a great day!"}),
                message: Some("Farewell acknowledged".to_string()),
            }),
            "status" => Ok(PluginOutput {
                success: true,
                data: serde_json::json!({
                    "status": "active",
                    "plugins_loaded": 1,
                    "uptime": "running"
                }),
                message: Some("System status retrieved".to_string()),
            }),
            _ => Ok(PluginOutput {
                success: false,
                data: serde_json::json!({"error": "Unknown command"}),
                message: Some(format!("Command '{}' not recognized", input.command)),
            }),
        }
    }

    async fn cleanup(&self) -> Result<()> {
        tracing::info!("Built-in commands plugin cleaned up");
        Ok(())
    }
}
