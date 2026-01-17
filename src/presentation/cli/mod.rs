// CLI presentation layer

use crate::application::services::VoiceCommandProcessor;
use crate::application::use_cases::{ProcessVoiceCommandUseCase, GetSystemStatusUseCase};
use crate::application::dtos::{VoiceCommandRequest, SystemStatusResponse};
use crate::shared::{Result, Error};
use std::sync::Arc;
use std::io::Write;
use tokio::io::{self, AsyncBufReadExt, BufReader};

pub struct CliInterface {
    voice_processor: Arc<VoiceCommandProcessor>,
    process_use_case: ProcessVoiceCommandUseCase,
    status_use_case: GetSystemStatusUseCase,
}

impl CliInterface {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        let process_use_case = ProcessVoiceCommandUseCase::new(voice_processor.clone());
        let status_use_case = GetSystemStatusUseCase::new(voice_processor.clone());

        Self {
            voice_processor,
            process_use_case,
            status_use_case,
        }
    }

    pub async fn run_interactive(&self) -> Result<()> {
        println!("🎤 Vibespeak CLI - Voice Command Interface");
        println!("==========================================");
        println!("Type voice commands or 'help' for commands, 'quit' to exit");
        println!();

        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        loop {
            print!("vibespeak> ");
            std::io::stdout().flush()?;

            match lines.next_line().await? {
                Some(line) => {
                    let command = line.trim();
                    if command.is_empty() {
                        continue;
                    }

                    match command {
                        "quit" | "exit" | "q" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" | "h" => {
                            self.show_help();
                        }
                        "status" | "s" => {
                            self.show_status().await?;
                        }
                        "list" | "l" => {
                            self.list_commands();
                        }
                        "plugins" | "p" => {
                            self.list_plugins();
                        }
                        _ => {
                            self.process_command(command).await?;
                        }
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  help, h     - Show this help");
        println!("  status, s   - Show system status");
        println!("  list, l     - List available commands");
        println!("  plugins, p  - List loaded plugins");
        println!("  quit, exit  - Exit the CLI");
        println!();
        println!("Voice commands:");
        println!("  hello       - Greeting");
        println!("  status      - System status");
        println!("  [custom]    - Any configured voice command");
        println!();
    }

    async fn show_status(&self) -> Result<()> {
        let status: SystemStatusResponse = self.status_use_case.execute().await?;

        println!("System Status:");
        println!("  Version: {}", status.version);
        println!("  Uptime: {}s", status.uptime);
        println!("  Active Plugins: {}", status.active_plugins);
        println!("  Loaded Commands: {}", status.loaded_commands);
        println!("  Active Workflows: {}", status.active_workflows);

        if let Some(memory) = status.memory_usage_mb {
            println!("  Memory Usage: {:.1} MB", memory);
        }
        if let Some(cpu) = status.cpu_usage_percent {
            println!("  CPU Usage: {:.1}%", cpu);
        }
        println!();

        Ok(())
    }

    fn list_commands(&self) {
        println!("Available voice commands:");
        // TODO: Get from command repository
        println!("  hello - Greeting response");
        println!("  status - System status");
        println!("  (More commands available via web interface)");
        println!();
    }

    fn list_plugins(&self) {
        let plugins = self.voice_processor.get_available_plugins();
        println!("Loaded plugins ({}):", plugins.len());

        for plugin in plugins {
            println!("  {} v{} - {}", plugin.name, plugin.version, plugin.description);
            println!("    Capabilities: {}", plugin.capabilities.join(", "));
        }
        println!();
    }

    async fn process_command(&self, command: &str) -> Result<()> {
        println!("Processing: \"{}\"", command);

        let request = VoiceCommandRequest {
            text: command.to_string(),
            voice: None,
            context: None,
        };

        match self.process_use_case.execute(request).await {
            Ok(response) => {
                if response.success {
                    println!("✅ Command executed successfully");
                    if let Some(msg) = response.message {
                        println!("  {}", msg);
                    }
                } else {
                    println!("❌ Command failed");
                    if let Some(msg) = response.message {
                        println!("  {}", msg);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
        println!();

        Ok(())
    }
}
