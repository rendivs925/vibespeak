mod config;
use config::CommandConfig;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use vosk::{Model, Recognizer};

const MODEL_PATH: &str = "model/vosk-model-small-en-us-0.15";
const COMMANDS_PATH: &str = "config/commands.toml";
const SILENCE_THRESHOLD: i16 = 60;

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

fn best_fuzzy_match<'a>(phrase: &str, commands: &'a HashSet<String>) -> Option<&'a str> {
    use strsim::jaro_winkler;
    let mut best: Option<(&str, f64)> = None;
    for cmd in commands.iter() {
        let sim = jaro_winkler(phrase, cmd);
        if sim > 0.91 {
            if let Some((_, best_sim)) = best {
                if sim > best_sim {
                    best = Some((cmd.as_str(), sim));
                }
            } else {
                best = Some((cmd.as_str(), sim));
            }
        }
    }
    best.map(|(cmd, _)| cmd)
}

fn typing_mode(model: &Model) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        terminal,
    };

    let _ = Command::new("notify-send")
        .arg("Voice Typing Mode")
        .arg("Voice typing is now ACTIVE. Press Esc or Win+T to exit.")
        .arg("-i")
        .arg("dialog-information")
        .arg("-t")
        .arg("3000")
        .spawn();

    println!("[INFO] Typing mode (dictation) ON. Press Esc to exit, or Win+T to toggle.");
    let mut recognizer = Recognizer::new(model, 16000.0).unwrap();
    let mut mic = start_rec().unwrap();
    let mut audio = mic.stdout.take().unwrap();
    let mut buffer = [0u8; 256];

    terminal::enable_raw_mode()?;
    let mut toggle_requested = false;
    let mut stop_reason = "[INFO] Exited typing mode.";

    'outer: loop {
        if event::poll(Duration::from_millis(2))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                if code == KeyCode::Esc {
                    stop_reason = "[INFO] Exiting typing mode (Esc key)";
                    break 'outer;
                }
                if code == KeyCode::Char('t') && modifiers.contains(KeyModifiers::SUPER) {
                    stop_reason = "[INFO] Toggling typing mode (Win+T)";
                    toggle_requested = true;
                    break 'outer;
                }
            }
        }

        if let Ok(n) = audio.read(&mut buffer) {
            if n == 0 {
                stop_reason = "[INFO] End of audio stream, exiting typing mode.";
                break;
            }
            let samples: Vec<i16> = buffer[..n]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            if samples.iter().all(|&x| x.abs() < SILENCE_THRESHOLD) {
                continue;
            }

            if recognizer.accept_waveform(&samples).unwrap() == vosk::DecodingState::Finalized {
                match recognizer.result() {
                    vosk::CompleteResult::Single(sr) => {
                        let text = sr.text.trim();
                        if !text.is_empty() && text != "type" {
                            let escaped = text.replace("'", r"'\''");
                            let send_cmd = format!("xdotool type '{}'", escaped);
                            let _ = Command::new("sh").arg("-c").arg(&send_cmd).spawn();
                        } else if text == "type" {
                            stop_reason =
                                "[INFO] Voice command 'type' detected, toggling typing mode.";
                            toggle_requested = true;
                            break 'outer;
                        }
                        print!("\r{: <80}\r", "");
                        std::io::stdout().flush().unwrap();
                    }
                    _ => {}
                }
                recognizer.reset();
            }
        }
    }
    terminal::disable_raw_mode()?;

    let _ = Command::new("notify-send")
        .arg("Voice Typing Mode")
        .arg("Voice typing is now OFF.")
        .arg("-i")
        .arg("dialog-information")
        .arg("-t")
        .arg("2000")
        .spawn();

    println!("{stop_reason}");
    if toggle_requested {
        return Ok(());
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

    let config = CommandConfig::load_from(COMMANDS_PATH)?;
    let command_map: HashMap<_, _> = config
        .commands
        .iter()
        .map(|(k, v)| (k.trim().to_lowercase(), v.clone()))
        .collect();

    let commands: HashSet<String> = command_map.keys().cloned().collect();
    let grammar: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();
    let model = Model::new(MODEL_PATH).unwrap();

    println!(
        "[INFO] Ready. Say a command or 'type' for typing mode. Win+T also toggles typing mode."
    );

    loop {
        let mut recognizer = Recognizer::new_with_grammar(&model, 16000.0, &grammar).unwrap();
        let mut mic = start_rec()?;
        let mut audio = mic.stdout.take().unwrap();
        let mut buffer = [0u8; 256];

        'main_listen: loop {
            if event::poll(Duration::from_millis(2))? {
                if let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = event::read()?
                {
                    if code == KeyCode::Char('t') && modifiers.contains(KeyModifiers::SUPER) {
                        println!("[INFO] Toggling typing mode (Win+T)");
                        typing_mode(&model)?;
                        println!("[INFO] Returning to listening mode.");
                        break 'main_listen;
                    }
                }
            }

            if let Ok(n) = audio.read(&mut buffer) {
                if n == 0 {
                    println!("[INFO] End of audio stream, restarting recognizer.");
                    break 'main_listen;
                }
                let samples: Vec<i16> = buffer[..n]
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();

                if samples.iter().all(|&x| x.abs() < SILENCE_THRESHOLD) {
                    continue;
                }

                if recognizer.accept_waveform(&samples).unwrap() == vosk::DecodingState::Finalized {
                    match recognizer.result() {
                        vosk::CompleteResult::Single(sr) => {
                            let text = sr.text.trim().to_lowercase();
                            if !text.is_empty() {
                                println!("[FINAL] Detected: \"{}\"", text);
                                if let Some(fuzzy_key) = best_fuzzy_match(&text, &commands) {
                                    if fuzzy_key == "type" {
                                        println!("[INFO] Voice command 'type' detected, entering typing mode.");
                                        typing_mode(&model)?;
                                        println!("[INFO] Returning to listening mode.");
                                        break 'main_listen;
                                    }
                                    if let Some(cmd) = command_map.get(fuzzy_key) {
                                        println!(
                                            "[EXEC] Fuzzy matched \"{}\" → \"{}\". Running: `{}`",
                                            text, fuzzy_key, cmd
                                        );
                                        let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    recognizer.reset();
                }
            }
        }
    }
}
