mod config;
use config::CommandConfig;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use vosk::{CompleteResult, DecodingState, Model, Recognizer};

const MODEL_PATH: &str = "model/vosk-model-small-en-us-0.15";
const COMMANDS_PATH: &str = "config/commands.toml";
const SILENCE_THRESHOLD: i16 = 120;

/// Start 'rec' process for microphone input
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

/// Typing mode: Say anything, it's typed. Esc = exit, Win+T = toggle typing mode.
fn typing_mode(model: &Model) -> anyhow::Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        terminal,
    };

    println!("Typing mode ON. Press Esc to exit, or Win+T to toggle.");
    let mut recognizer = Recognizer::new(model, 16000.0).unwrap();
    let mut mic = start_rec().unwrap();
    let mut audio = mic.stdout.take().unwrap();
    let mut buffer = [0u8; 1024];

    terminal::enable_raw_mode()?;

    let mut toggle_requested = false;
    'outer: loop {
        // Keyboard event check: Esc to exit, Win+T to toggle (same as voice "type")
        if event::poll(Duration::from_millis(20))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                if code == KeyCode::Esc {
                    println!("Exiting typing mode (Esc)");
                    break 'outer;
                }
                // Win+T (Super+T) toggles typing mode
                if code == KeyCode::Char('t') && modifiers.contains(KeyModifiers::SUPER) {
                    println!("Toggling typing mode (Win+T)");
                    toggle_requested = true;
                    break 'outer;
                }
            }
        }
        if let Ok(n) = audio.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let samples: Vec<i16> = buffer[..n]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();

            if samples.iter().all(|&x| x.abs() < SILENCE_THRESHOLD) {
                continue;
            }

            if recognizer.accept_waveform(&samples).unwrap() == DecodingState::Finalized {
                match recognizer.result() {
                    CompleteResult::Single(sr) => {
                        let text = sr.text.trim();
                        // If the user says "type", treat it as toggle
                        if text == "type" {
                            println!("Toggling typing mode (voice)");
                            toggle_requested = true;
                            break 'outer;
                        }
                        if !text.is_empty() {
                            let escaped = text.replace("'", r"'\''");
                            let send_cmd = format!("xdotool type '{}'", escaped);
                            println!("Typing: '{}'", escaped);
                            let _ = Command::new("sh").arg("-c").arg(&send_cmd).spawn();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    terminal::disable_raw_mode()?;
    if toggle_requested {
        // Go back to listening and re-enter typing mode
        return Ok(());
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let config = CommandConfig::load_from(COMMANDS_PATH)?;
    let command_map: HashMap<_, _> = config
        .commands
        .iter()
        .map(|(k, v)| (k.trim().to_lowercase(), v.clone()))
        .collect();

    let mut commands: HashSet<String> = command_map.keys().cloned().collect();
    commands.insert("type".into()); // only one trigger needed

    let grammar: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();
    let model = Model::new(MODEL_PATH).unwrap();

    println!("Ready. Say a command or 'type' for typing mode. Win+T also toggles typing mode.");

    let mut in_typing_mode = false;

    loop {
        if in_typing_mode {
            // Enter typing mode. If toggled, flip flag and continue main loop.
            typing_mode(&model)?;
            in_typing_mode = false;
            println!("Exited typing mode.");
            continue;
        }

        let mut recognizer = Recognizer::new_with_grammar(&model, 16000.0, &grammar).unwrap();
        let mut mic = start_rec()?;
        let mut audio = mic.stdout.take().unwrap();
        let mut buffer = [0u8; 1024];

        let mut last_partial = String::new();
        let mut prefix_matched = false;
        let mut prefix_start_time = Instant::now();

        'listen: loop {
            // Keyboard event check for Win+T even when not in typing mode
            use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
            if event::poll(Duration::from_millis(20))? {
                if let Event::Key(KeyEvent {
                    code, modifiers, ..
                }) = event::read()?
                {
                    if code == KeyCode::Char('t') && modifiers.contains(KeyModifiers::SUPER) {
                        println!("Toggling typing mode (Win+T)");
                        in_typing_mode = true;
                        break 'listen;
                    }
                }
            }
            if let Ok(n) = audio.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                let samples: Vec<i16> = buffer[..n]
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();

                if samples.iter().all(|&x| x.abs() < SILENCE_THRESHOLD) {
                    continue;
                }

                recognizer.accept_waveform(&samples).unwrap();
                let partial = recognizer.partial_result().partial.trim().to_lowercase();

                if !partial.is_empty() && partial != last_partial {
                    last_partial = partial.clone();

                    if partial == "type" {
                        in_typing_mode = true;
                        break 'listen;
                    }

                    if let Some(cmd) = command_map.get(&partial) {
                        println!("Matched '{}': Running `{}`", partial, cmd);
                        let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
                        break 'listen;
                    }

                    prefix_matched = grammar.iter().any(|c| c.starts_with(&partial));
                    if prefix_matched {
                        prefix_start_time = Instant::now();
                    } else if !partial.is_empty() {
                        println!("No command starts with '{}', resetting.", partial);
                        break 'listen;
                    }
                }

                if prefix_matched && prefix_start_time.elapsed() >= Duration::from_millis(400) {
                    println!(
                        "Prefix '{}' incomplete after 400ms, resetting.",
                        last_partial
                    );
                    break 'listen;
                }
            }
        }
    }
}
