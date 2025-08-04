# vibespeak

Vibespeak is a CLI tool for hands-free, voice-driven automation of your Linux desktop. It listens to your speech and executes mapped system commands in real time, making it ideal for users of i3, tmux, Alacritty, Neovim, and terminal-centric workflows.

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

## Quick Start

1. **Install dependencies:**  
   (for Ubuntu/Debian, adapt as needed for your distro)

   ```sh
   sudo apt install xdotool i3-wm sox wget unzip libasound2-dev libstdc++-12-dev
   ```

2. **Clone this repository:**

   ```sh
   git clone https://github.com/rendivs925/vibespeak.git
   cd vibespeak
   ```

3. **Run Vibespeak:**

   - Using cargo directly (**recommended for development and first run**):

     ```sh
     cargo run --release
     ```

   - Or install locally (**useful for repeated use**):

     ```sh
     cargo install --path .
     vibespeak
     ```

---

## Usage

Vibespeak will listen for voice commands and execute the mapped system actions.
To customize your commands, edit `config/commands.toml` in the project directory.

---

## Example `commands.toml`

```toml
# --- Tmux Control (prefix is C-a) ---
"split pane"         = "xdotool key ctrl+a bar"
"vertical split"     = "xdotool key ctrl+a minus"
"next pane"          = "xdotool key ctrl+a o"
"pane zero"          = "xdotool key ctrl+a 0"

# --- i3 Window Manager ---
"workspace one"      = "i3-msg workspace 1"
"move left"          = "i3-msg focus left"

# --- General Shell ---
"clear"              = "xdotool type 'clear'; xdotool key Return"
"zoom in"            = "xdotool key ctrl+KP_Add"
```

---

## Customizing

1. Open `config/commands.toml` in your preferred editor.
2. Add, edit, or remove any command-to-action mapping.
3. Restart Vibespeak for changes to take effect.

---

## Requirements

- Linux (tested on modern desktop environments)
- A working microphone
- The following utilities installed: `xdotool`, `i3-msg`, `sox`, `wget`, `unzip`, `libasound2-dev`, `libstdc++-12-dev`
- [Vosk model](https://alphacephei.com/vosk/models) (see above for setup)

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
