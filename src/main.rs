mod config;
use config::CommandConfig;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use vosk::{Model, Recognizer};

const MODEL_PATH: &str = "model/vosk-model-small-en-us-0.15";
const COMMANDS_PATH: &str = "config/commands.toml";

fn start_rec() -> std::io::Result<std::process::Child> {
    Command::new("rec")
        .args(&[
            "--no-show-progress",
            "--clobber",
            "-q",
            "-r",
            "16000",
            "-c",
            "1",
            "-b",
            "16",
            "-e",
            "signed-integer",
            "-t",
            "raw",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
}

fn is_prefix_of_any_command(prefix: &str, commands: &[String]) -> bool {
    commands.iter().any(|cmd| cmd.starts_with(prefix))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CommandConfig::load_from(COMMANDS_PATH)
        .map_err(|e| format!("Failed to load config: {e}"))?;

    let model = Model::new(MODEL_PATH)
        .ok_or(format!(
            "Failed to load Vosk model at '{}'.\nPlease follow the instructions in the README to download and place the model.",
            MODEL_PATH
        ))?;

    let commands: Vec<String> = config.commands.keys().map(|s| s.to_lowercase()).collect();
    let grammar: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();

    println!(
        "Ready and listening for commands (model: {})...",
        MODEL_PATH
    );

    loop {
        let mut recognizer = Recognizer::new_with_grammar(&model, 16000.0, &grammar)
            .ok_or("Failed to create recognizer")?;

        let mut mic = match start_rec() {
            Ok(mic) => mic,
            Err(e) => {
                eprintln!("Microphone failed to start: {e}");
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        let mut audio = mic.stdout.take().expect("No mic output");
        let mut buffer = [0u8; 4096];

        let mut last_partial = String::new();
        let mut last_speech = Instant::now();

        loop {
            if let Ok(n) = audio.read(&mut buffer) {
                if n == 0 {
                    break;
                }

                let samples: Vec<i16> = buffer[..n]
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();

                recognizer.accept_waveform(&samples)?;

                let partial = recognizer.partial_result().partial.trim().to_lowercase();

                if !partial.is_empty() && partial != last_partial {
                    last_speech = Instant::now();
                    last_partial = partial.clone();

                    if let Some(cmd) = config.commands.get(partial.as_str()) {
                        println!("Matched '{}': Running `{}`", partial, cmd);
                        if let Err(e) = Command::new("sh").arg("-c").arg(cmd).spawn() {
                            eprintln!("Failed to run command: {e}");
                        }

                        // Enter dictation mode after "neo find"
                        if partial == "neo find" {
                            println!(
                                "Dictation mode: Please say your Neovim search (max 5 seconds)..."
                            );
                            let mut free_recognizer = Recognizer::new(&model, 16000.0)
                                .ok_or("Failed to create free-text recognizer")?;
                            let mut search_text = String::new();
                            let dictation_start = Instant::now();

                            // Dictation loop (max 5 sec or until something is spoken)
                            while dictation_start.elapsed() < Duration::from_secs(5) {
                                if let Ok(n) = audio.read(&mut buffer) {
                                    if n == 0 {
                                        break;
                                    }
                                    let samples: Vec<i16> = buffer[..n]
                                        .chunks_exact(2)
                                        .map(|b| i16::from_le_bytes([b[0], b[1]]))
                                        .collect();
                                    free_recognizer.accept_waveform(&samples)?;
                                    let text =
                                        free_recognizer.partial_result().partial.trim().to_string();
                                    if !text.is_empty() && text != search_text {
                                        search_text = text;
                                        println!("Heard search: '{}'", search_text);
                                    }
                                }
                            }

                            if !search_text.is_empty() {
                                // Send the search text as keystrokes to Neovim
                                let escaped = search_text.replace("'", r"'\''"); // escape single quotes
                                let send_cmd = format!("xdotool type '{}'", escaped);
                                println!("Typing into Neovim: {}", escaped);
                                if let Err(e) = Command::new("sh").arg("-c").arg(&send_cmd).spawn()
                                {
                                    eprintln!("Failed to type search query: {e}");
                                }
                                // Send Return to execute search
                                if let Err(e) =
                                    Command::new("xdotool").arg("key").arg("Return").spawn()
                                {
                                    eprintln!("Failed to send Return: {e}");
                                }
                            } else {
                                println!("No search text heard.");
                            }

                            recognizer.reset();
                            last_partial.clear();
                            continue;
                        }

                        recognizer.reset();
                        last_partial.clear();
                        continue;
                    }

                    if !is_prefix_of_any_command(&partial, &commands) {
                        println!("No command or prefix match for '{}', resetting.", partial);
                        recognizer.reset();
                        last_partial.clear();
                        continue;
                    }
                }

                if !last_partial.is_empty() && last_speech.elapsed() >= Duration::from_millis(1200)
                {
                    println!("Silence after prefix '{}', resetting.", last_partial);
                    recognizer.reset();
                    last_partial.clear();
                }
            }
        }

        println!("Restarting microphone...");
    }
}
