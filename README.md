# vibespeak

**Vibespeak** is a CLI tool for hands-free, voice-driven automation of your Linux desktop.
It listens to your speech and executes mapped system commands in real time, making it ideal for users of i3, tmux, Alacritty, Neovim, and terminal-centric workflows.

---

## Features

- Voice-activated command execution
- Integration with i3 window manager for workspace and window management
- Tmux pane and window control
- Font zoom and management for Alacritty
- Neovim automation for common editing commands
- Customizable: add or modify your own command mappings
- Compatible with any shell or terminal

---

## How It Works

Vibespeak maps spoken phrases to shell commands using a simple `[commands]` configuration.
When you say a registered command (such as “split pane” or “workspace one”), Vibespeak runs the corresponding action using utilities like `xdotool` and `i3-msg`.

**Example workflow:**

- “split pane” &rarr; splits your current tmux window
- “workspace one” &rarr; moves i3 to workspace 1
- “zoom in” &rarr; increases font size in Alacritty
- “save file” &rarr; saves the current file in Neovim
- “clear” &rarr; clears your terminal

See [`commands.toml`](./commands.toml) for a ready-made set of mappings.

---

## Installation

1. **Install Rust (if needed):**
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Install system dependencies:**
   On most Linux systems:

   ```sh
   sudo apt install libstdc++-12-dev libasound2-dev
   ```

   You may also need a Vosk model for speech recognition.
   Download a model and place it in your project folder as `model/`.

3. **Install vibespeak from source:**
   ```sh
   git clone https://github.com/rendivs925/vibespeak.git
   cd vibespeak
   cargo install --path .
   ```

---

## Usage

Start Vibespeak in your terminal:

```sh
vibespeak
```

It will listen for voice commands and execute the mapped system actions.
To customize your commands, edit `commands.toml` in the project directory.

---

## Example `commands.toml`

```toml
# --- Tmux Control (prefix is C-a) ---
"split pane"         = "xdotool key ctrl+a bar"
"vertical split"     = "xdotool key ctrl+a minus"
"next pane"          = "xdotool key ctrl+a o"
"pane zero"          = "xdotool key ctrl+a 0"
# ...more commands

# --- i3 Window Manager ---
"workspace one"      = "i3-msg workspace 1"
"move left"          = "i3-msg focus left"
# ...more commands

# --- General Shell ---
"clear"              = "xdotool type 'clear'; xdotool key Return"
"zoom in"            = "xdotool key ctrl+KP_Add"
```

---

## Customizing

1. Open `commands.toml` in your preferred editor.
2. Add, edit, or remove any command-to-action mapping.
3. Restart Vibespeak for changes to take effect.

---

## Requirements

- Linux (tested on modern desktop environments)
- A working microphone
- `xdotool`, `i3-msg`, and any other system utilities referenced in your command mappings
- [Vosk](https://alphacephei.com/vosk) speech recognition model

---

## License

This project is licensed under the MIT or Apache-2.0 license.

---

## Contributing

Contributions, suggestions, and pull requests are welcome.
If you encounter a problem or have a feature request, please open an issue on [GitHub](https://github.com/rendivs925/vibespeak).

---

## Acknowledgments

- [Vosk](https://alphacephei.com/vosk) for fast, offline speech recognition
- The Linux open source community for i3, tmux, Alacritty, and more
