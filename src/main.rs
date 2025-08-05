mod config;

use config::CommandConfig;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use vosk::{CompleteResult, DecodingState, Model, Recognizer};

const MODEL_PATH: &str = "model/vosk-model-small-en-us-0.15";
const COMMANDS_PATH: &str = "config/commands.toml";

fn start_rec() -> std::io::Result<std::process::Child> {
    Command::new("rec")
        .args(&[
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

fn typing_mode(model: &Model) {
    println!("Typing mode ON. Say 'type off' to exit.");
    let mut recognizer = Recognizer::new(model, 16000.0).unwrap();
    let mut mic = start_rec().unwrap();
    let mut audio = mic.stdout.take().unwrap();
    let mut buffer = [0u8; 4096];

    loop {
        if let Ok(n) = audio.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let samples: Vec<i16> = buffer[..n]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();

            if recognizer.accept_waveform(&samples).unwrap() == DecodingState::Finalized {
                match recognizer.result() {
                    CompleteResult::Single(sr) => {
                        let text = sr.text.trim();
                        if text == "type off" {
                            println!("Typing mode OFF.");
                            break;
                        }
                        if !text.is_empty() {
                            let escaped = text.replace("'", r"'\''");
                            let send_cmd = format!("xdotool type '{}'", escaped);
                            println!("Typing: '{}'", escaped);
                            if let Err(e) = Command::new("sh").arg("-c").arg(&send_cmd).spawn() {
                                eprintln!("Failed to type: {e}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let config = CommandConfig::load_from(COMMANDS_PATH)?;
    let grammar: Vec<&str> = config.commands.keys().map(|s| s.as_str()).collect();
    let model = Model::new(MODEL_PATH).unwrap();

    println!("Ready. Say a command or 'type on' for typing mode...");

    loop {
        let mut recognizer = Recognizer::new_with_grammar(&model, 16000.0, &grammar).unwrap();
        let mut mic = start_rec()?;
        let mut audio = mic.stdout.take().unwrap();
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
                    last_partial = partial.clone();
                    last_speech = Instant::now();

                    if partial == "type on" {
                        typing_mode(&model);
                        break;
                    }

                    if let Some(cmd) = config.commands.get(&partial) {
                        if !cmd.is_empty() {
                            println!("Matched '{}': Running `{}`", partial, cmd);
                            if let Err(e) = Command::new("sh").arg("-c").arg(cmd).spawn() {
                                eprintln!("Failed to run command: {e}");
                            }
                        }
                    }
                }

                if recognizer.accept_waveform(&samples).unwrap() == DecodingState::Finalized {
                    match recognizer.result() {
                        CompleteResult::Single(sr) => {
                            let text = sr.text.trim();
                            if !text.is_empty() {
                                println!("Final recognized: {}", text);
                            }
                        }
                        _ => {}
                    }
                }

                if !last_partial.is_empty() && last_speech.elapsed() >= Duration::from_millis(1200)
                {
                    last_partial.clear();
                    recognizer.reset();
                }
            }
        }
    }
}
