# Vibespeak Dictation Browser Extension

This browser extension enables seamless dictation capabilities for Vibespeak, allowing you to type anywhere on the web without using a physical keyboard.

## Features

- 🎤 **Voice Dictation**: Start dictation in any text input field
- 📝 **Smart Text Insertion**: Automatically inserts dictated text into focused inputs
- 🎯 **Multi-Input Support**: Works with regular inputs, textareas, contenteditable elements, and code editors
- ⌨️ **Keyboard Shortcuts**: Use Ctrl+Shift+D to toggle dictation
- 🔄 **Real-time Feedback**: Visual indicators show dictation status

## Installation

### Chrome/Edge
1. Open Chrome/Edge and go to `chrome://extensions/`
2. Enable "Developer mode" in the top right
3. Click "Load unpacked" and select the `web/extension/` folder
4. The extension should now appear in your extensions list

### Firefox
1. Open Firefox and go to `about:debugging`
2. Click "This Firefox" in the sidebar
3. Click "Load Temporary Add-on"
4. Select the `manifest.json` file from the `web/extension/` folder

## Usage

### Basic Dictation
1. Click the Vibespeak Dictation extension icon in your browser toolbar
2. Click "Start Dictation" or use the keyboard shortcut Ctrl+Shift+D
3. Focus on any text input field (regular inputs, textareas, contenteditable areas, code editors)
4. Start speaking - your words will be transcribed in real-time
5. Click "Stop Dictation" when finished

### Automatic Text Insertion
The extension automatically detects when you're focused on a text input and shows a dictation indicator. Dictated text can be inserted directly into the focused field.

### Supported Input Types
- Regular `<input>` elements
- `<textarea>` elements
- Contenteditable elements (like rich text editors)
- Code editors (Monaco, Ace, CodeMirror)
- Custom input fields with proper accessibility attributes

## Technical Details

### Architecture
- **Content Script**: Injects dictation capabilities into web pages
- **Background Script**: Manages extension lifecycle and cross-tab communication
- **Popup Interface**: Provides user controls and status display
- **Web API Integration**: Communicates with the main Vibespeak server

### Permissions
- `activeTab`: Access the currently active tab for dictation
- `storage`: Store user preferences and settings
- `scripting`: Inject content scripts for dictation functionality
- `host_permissions`: Communicate with the local Vibespeak server

### Keyboard Shortcuts
- `Ctrl+Shift+D`: Toggle dictation mode
- Works globally when a compatible input field is focused

## Troubleshooting

### Dictation Not Working
1. Ensure the Vibespeak server is running on `http://localhost:8080`
2. Check that the extension has the necessary permissions
3. Verify your microphone permissions in the browser
4. Try refreshing the page and reloading the extension

### Text Not Inserting
1. Make sure you're focused on a text input field
2. Check the browser console for error messages
3. Ensure the target application supports text input events

### Extension Not Loading
1. Verify the manifest.json file is valid
2. Check that all required files are present in the extension folder
3. Try reloading the extension in the browser's extension manager

## Development

### File Structure
```
web/extension/
├── manifest.json      # Extension configuration
├── background.js      # Background service worker
├── content.js         # Content script for web pages
├── popup.html         # Extension popup interface
├── popup.js           # Popup functionality
└── icons/             # Extension icons (16x16, 32x32, 48x48, 128x128)
```

### Building for Production
1. Update version numbers in `manifest.json`
2. Test all functionality thoroughly
3. Package the extension for distribution
4. Submit to browser extension stores if desired

## Contributing

When adding new features:
1. Test across different input field types
2. Ensure proper error handling
3. Update documentation
4. Follow the existing code style and patterns

## License

This extension is part of the Vibespeak project. See the main project license for details.