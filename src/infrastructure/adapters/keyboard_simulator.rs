use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, EventType, InputEvent, Key,
};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;
use tracing;

static KEY_MAP: OnceLock<HashMap<char, (Key, bool)>> = OnceLock::new();

fn get_key_map() -> &'static HashMap<char, (Key, bool)> {
    KEY_MAP.get_or_init(|| {
        let mut map = HashMap::new();

        // Lowercase letters
        map.insert('a', (Key::KEY_A, false));
        map.insert('b', (Key::KEY_B, false));
        map.insert('c', (Key::KEY_C, false));
        map.insert('d', (Key::KEY_D, false));
        map.insert('e', (Key::KEY_E, false));
        map.insert('f', (Key::KEY_F, false));
        map.insert('g', (Key::KEY_G, false));
        map.insert('h', (Key::KEY_H, false));
        map.insert('i', (Key::KEY_I, false));
        map.insert('j', (Key::KEY_J, false));
        map.insert('k', (Key::KEY_K, false));
        map.insert('l', (Key::KEY_L, false));
        map.insert('m', (Key::KEY_M, false));
        map.insert('n', (Key::KEY_N, false));
        map.insert('o', (Key::KEY_O, false));
        map.insert('p', (Key::KEY_P, false));
        map.insert('q', (Key::KEY_Q, false));
        map.insert('r', (Key::KEY_R, false));
        map.insert('s', (Key::KEY_S, false));
        map.insert('t', (Key::KEY_T, false));
        map.insert('u', (Key::KEY_U, false));
        map.insert('v', (Key::KEY_V, false));
        map.insert('w', (Key::KEY_W, false));
        map.insert('x', (Key::KEY_X, false));
        map.insert('y', (Key::KEY_Y, false));
        map.insert('z', (Key::KEY_Z, false));

        // Uppercase letters (need shift)
        map.insert('A', (Key::KEY_A, true));
        map.insert('B', (Key::KEY_B, true));
        map.insert('C', (Key::KEY_C, true));
        map.insert('D', (Key::KEY_D, true));
        map.insert('E', (Key::KEY_E, true));
        map.insert('F', (Key::KEY_F, true));
        map.insert('G', (Key::KEY_G, true));
        map.insert('H', (Key::KEY_H, true));
        map.insert('I', (Key::KEY_I, true));
        map.insert('J', (Key::KEY_J, true));
        map.insert('K', (Key::KEY_K, true));
        map.insert('L', (Key::KEY_L, true));
        map.insert('M', (Key::KEY_M, true));
        map.insert('N', (Key::KEY_N, true));
        map.insert('O', (Key::KEY_O, true));
        map.insert('P', (Key::KEY_P, true));
        map.insert('Q', (Key::KEY_Q, true));
        map.insert('R', (Key::KEY_R, true));
        map.insert('S', (Key::KEY_S, true));
        map.insert('T', (Key::KEY_T, true));
        map.insert('U', (Key::KEY_U, true));
        map.insert('V', (Key::KEY_V, true));
        map.insert('W', (Key::KEY_W, true));
        map.insert('X', (Key::KEY_X, true));
        map.insert('Y', (Key::KEY_Y, true));
        map.insert('Z', (Key::KEY_Z, true));

        // Numbers
        map.insert('0', (Key::KEY_0, false));
        map.insert('1', (Key::KEY_1, false));
        map.insert('2', (Key::KEY_2, false));
        map.insert('3', (Key::KEY_3, false));
        map.insert('4', (Key::KEY_4, false));
        map.insert('5', (Key::KEY_5, false));
        map.insert('6', (Key::KEY_6, false));
        map.insert('7', (Key::KEY_7, false));
        map.insert('8', (Key::KEY_8, false));
        map.insert('9', (Key::KEY_9, false));

        // Shifted numbers (symbols)
        map.insert('!', (Key::KEY_1, true));
        map.insert('@', (Key::KEY_2, true));
        map.insert('#', (Key::KEY_3, true));
        map.insert('$', (Key::KEY_4, true));
        map.insert('%', (Key::KEY_5, true));
        map.insert('^', (Key::KEY_6, true));
        map.insert('&', (Key::KEY_7, true));
        map.insert('*', (Key::KEY_8, true));
        map.insert('(', (Key::KEY_9, true));
        map.insert(')', (Key::KEY_0, true));

        // Common punctuation
        map.insert(' ', (Key::KEY_SPACE, false));
        map.insert('\n', (Key::KEY_ENTER, false));
        map.insert('\t', (Key::KEY_TAB, false));
        map.insert('.', (Key::KEY_DOT, false));
        map.insert(',', (Key::KEY_COMMA, false));
        map.insert(';', (Key::KEY_SEMICOLON, false));
        map.insert('\'', (Key::KEY_APOSTROPHE, false));
        map.insert('[', (Key::KEY_LEFTBRACE, false));
        map.insert(']', (Key::KEY_RIGHTBRACE, false));
        map.insert('\\', (Key::KEY_BACKSLASH, false));
        map.insert('`', (Key::KEY_GRAVE, false));
        map.insert('-', (Key::KEY_MINUS, false));
        map.insert('=', (Key::KEY_EQUAL, false));
        map.insert('/', (Key::KEY_SLASH, false));

        // Shifted punctuation
        map.insert('>', (Key::KEY_DOT, true));
        map.insert('<', (Key::KEY_COMMA, true));
        map.insert(':', (Key::KEY_SEMICOLON, true));
        map.insert('"', (Key::KEY_APOSTROPHE, true));
        map.insert('{', (Key::KEY_LEFTBRACE, true));
        map.insert('}', (Key::KEY_RIGHTBRACE, true));
        map.insert('|', (Key::KEY_BACKSLASH, true));
        map.insert('~', (Key::KEY_GRAVE, true));
        map.insert('_', (Key::KEY_MINUS, true));
        map.insert('+', (Key::KEY_EQUAL, true));
        map.insert('?', (Key::KEY_SLASH, true));

        map
    })
}

pub struct KeyboardSimulator {
    device: VirtualDevice,
}

impl KeyboardSimulator {
    pub fn new() -> Result<Self, String> {
        let mut keys = AttributeSet::<Key>::new();

        // Add all keys we might need
        keys.insert(Key::KEY_A);
        keys.insert(Key::KEY_B);
        keys.insert(Key::KEY_C);
        keys.insert(Key::KEY_D);
        keys.insert(Key::KEY_E);
        keys.insert(Key::KEY_F);
        keys.insert(Key::KEY_G);
        keys.insert(Key::KEY_H);
        keys.insert(Key::KEY_I);
        keys.insert(Key::KEY_J);
        keys.insert(Key::KEY_K);
        keys.insert(Key::KEY_L);
        keys.insert(Key::KEY_M);
        keys.insert(Key::KEY_N);
        keys.insert(Key::KEY_O);
        keys.insert(Key::KEY_P);
        keys.insert(Key::KEY_Q);
        keys.insert(Key::KEY_R);
        keys.insert(Key::KEY_S);
        keys.insert(Key::KEY_T);
        keys.insert(Key::KEY_U);
        keys.insert(Key::KEY_V);
        keys.insert(Key::KEY_W);
        keys.insert(Key::KEY_X);
        keys.insert(Key::KEY_Y);
        keys.insert(Key::KEY_Z);
        keys.insert(Key::KEY_0);
        keys.insert(Key::KEY_1);
        keys.insert(Key::KEY_2);
        keys.insert(Key::KEY_3);
        keys.insert(Key::KEY_4);
        keys.insert(Key::KEY_5);
        keys.insert(Key::KEY_6);
        keys.insert(Key::KEY_7);
        keys.insert(Key::KEY_8);
        keys.insert(Key::KEY_9);
        keys.insert(Key::KEY_SPACE);
        keys.insert(Key::KEY_ENTER);
        keys.insert(Key::KEY_TAB);
        keys.insert(Key::KEY_BACKSPACE);
        keys.insert(Key::KEY_DOT);
        keys.insert(Key::KEY_COMMA);
        keys.insert(Key::KEY_SEMICOLON);
        keys.insert(Key::KEY_APOSTROPHE);
        keys.insert(Key::KEY_LEFTBRACE);
        keys.insert(Key::KEY_RIGHTBRACE);
        keys.insert(Key::KEY_BACKSLASH);
        keys.insert(Key::KEY_GRAVE);
        keys.insert(Key::KEY_MINUS);
        keys.insert(Key::KEY_EQUAL);
        keys.insert(Key::KEY_SLASH);
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_RIGHTSHIFT);
        keys.insert(Key::KEY_LEFTCTRL);
        keys.insert(Key::KEY_LEFTALT);
        keys.insert(Key::KEY_ESC);
        keys.insert(Key::KEY_DELETE);
        keys.insert(Key::KEY_HOME);
        keys.insert(Key::KEY_END);
        keys.insert(Key::KEY_PAGEUP);
        keys.insert(Key::KEY_PAGEDOWN);
        keys.insert(Key::KEY_UP);
        keys.insert(Key::KEY_DOWN);
        keys.insert(Key::KEY_LEFT);
        keys.insert(Key::KEY_RIGHT);
        keys.insert(Key::KEY_F1);
        keys.insert(Key::KEY_F2);
        keys.insert(Key::KEY_F3);
        keys.insert(Key::KEY_F4);
        keys.insert(Key::KEY_F5);
        keys.insert(Key::KEY_F6);
        keys.insert(Key::KEY_F7);
        keys.insert(Key::KEY_F8);
        keys.insert(Key::KEY_F9);
        keys.insert(Key::KEY_F10);
        keys.insert(Key::KEY_F11);
        keys.insert(Key::KEY_F12);

        let device = VirtualDeviceBuilder::new()
            .map_err(|e| format!("Failed to create virtual device builder: {}", e))?
            .name("Vibespeak Virtual Keyboard")
            .with_keys(&keys)
            .map_err(|e| format!("Failed to add keys to virtual device: {}", e))?
            .build()
            .map_err(|e| format!("Failed to build virtual device: {}", e))?;

        // Give the system time to recognize the new device
        thread::sleep(Duration::from_millis(100));

        tracing::info!("Created virtual keyboard device");

        Ok(Self { device })
    }

    fn press_key(&mut self, key: Key) -> Result<(), String> {
        let events = [
            InputEvent::new(EventType::KEY, key.code(), 1), // Key down
        ];
        self.device
            .emit(&events)
            .map_err(|e| format!("Failed to emit key down: {}", e))
    }

    fn release_key(&mut self, key: Key) -> Result<(), String> {
        let events = [
            InputEvent::new(EventType::KEY, key.code(), 0), // Key up
        ];
        self.device
            .emit(&events)
            .map_err(|e| format!("Failed to emit key up: {}", e))
    }

    fn tap_key(&mut self, key: Key) -> Result<(), String> {
        self.press_key(key)?;
        thread::sleep(Duration::from_micros(500));
        self.release_key(key)?;
        Ok(())
    }

    fn type_char(&mut self, ch: char) -> Result<(), String> {
        let key_map = get_key_map();

        if let Some(&(key, needs_shift)) = key_map.get(&ch) {
            if needs_shift {
                self.press_key(Key::KEY_LEFTSHIFT)?;
                thread::sleep(Duration::from_micros(200));
            }

            self.tap_key(key)?;

            if needs_shift {
                thread::sleep(Duration::from_micros(200));
                self.release_key(Key::KEY_LEFTSHIFT)?;
            }

            Ok(())
        } else {
            tracing::warn!("Unsupported character: '{}' (U+{:04X})", ch, ch as u32);
            Ok(()) // Skip unsupported characters
        }
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        tracing::info!("Typing {} characters via uinput", text.len());

        for ch in text.chars() {
            self.type_char(ch)?;
            // Small delay between keystrokes to simulate natural typing
            thread::sleep(Duration::from_millis(5));
        }

        Ok(())
    }

    pub fn press_key_combo(&mut self, keys: &[Key]) -> Result<(), String> {
        // Press all keys
        for &key in keys {
            self.press_key(key)?;
            thread::sleep(Duration::from_micros(200));
        }

        thread::sleep(Duration::from_millis(10));

        // Release all keys in reverse order
        for &key in keys.iter().rev() {
            self.release_key(key)?;
            thread::sleep(Duration::from_micros(200));
        }

        Ok(())
    }
}

pub fn type_text_uinput(text: &str) -> Result<(), String> {
    let mut simulator = KeyboardSimulator::new()?;
    simulator.type_text(text)
}

pub fn press_key_combo_uinput(keys: &[Key]) -> Result<(), String> {
    let mut simulator = KeyboardSimulator::new()?;
    simulator.press_key_combo(keys)
}

pub fn send_backspaces_uinput(count: usize) -> Result<(), String> {
    let mut simulator = KeyboardSimulator::new()?;
    for _ in 0..count {
        simulator.tap_key(Key::KEY_BACKSPACE)?;
        // Small delay between backspaces
        thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
