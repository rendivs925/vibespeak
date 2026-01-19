// Vibespeak Dictation Content Script
// Handles text injection into various input fields and text areas

class DictationManager {
    constructor() {
        this.isDictationActive = false;
        this.currentInput = null;
        this.dictationServer = 'http://localhost:8080';

        this.init();
    }

    init() {
        // Listen for messages from the extension
        chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
            this.handleMessage(request, sender, sendResponse);
            return true; // Keep message channel open for async responses
        });

        // Listen for keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            // Ctrl+Shift+D to toggle dictation
            if (e.ctrlKey && e.shiftKey && e.key === 'D') {
                e.preventDefault();
                this.toggleDictation();
            }
        });

        // Auto-detect input focus for dictation
        document.addEventListener('focusin', (e) => {
            if (this.isInputElement(e.target)) {
                this.currentInput = e.target;
                this.showDictationIndicator(e.target);
            }
        });

        document.addEventListener('focusout', (e) => {
            if (e.target === this.currentInput) {
                this.hideDictationIndicator();
                this.currentInput = null;
            }
        });
    }

    handleMessage(request, sender, sendResponse) {
        switch (request.action) {
            case 'startDictation':
                this.startDictation(request.inputType);
                sendResponse({ success: true });
                break;

            case 'stopDictation':
                this.stopDictation();
                sendResponse({ success: true });
                break;

            case 'insertText':
                this.insertText(request.text, request.inputType);
                sendResponse({ success: true });
                break;

            case 'getActiveElement':
                sendResponse({
                    element: this.getElementInfo(document.activeElement),
                    isInput: this.isInputElement(document.activeElement)
                });
                break;

            default:
                sendResponse({ error: 'Unknown action' });
        }
    }

    async startDictation(inputType) {
        this.isDictationActive = true;

        try {
            // Notify the dictation server
            const response = await fetch(`${this.dictationServer}/api/dictation/start`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    inputType: inputType || 'text',
                    url: window.location.href,
                    elementInfo: this.currentInput ? this.getElementInfo(this.currentInput) : null
                })
            });

            if (response.ok) {
                this.showDictationActiveIndicator();
                this.notifyExtension({ action: 'dictationStarted', inputType });
            }
        } catch (error) {
            console.error('Failed to start dictation:', error);
        }
    }

    async stopDictation() {
        this.isDictationActive = false;

        try {
            // Notify the dictation server
            await fetch(`${this.dictationServer}/api/dictation/stop`, {
                method: 'POST'
            });
        } catch (error) {
            console.error('Failed to stop dictation:', error);
        }

        this.hideDictationActiveIndicator();
        this.notifyExtension({ action: 'dictationStopped' });
    }

    insertText(text, inputType) {
        const target = this.currentInput || document.activeElement;

        if (!target) {
            console.warn('No active input element for text insertion');
            return;
        }

        switch (inputType) {
            case 'contenteditable':
                this.insertIntoContentEditable(target, text);
                break;
            case 'textarea':
            case 'input':
                this.insertIntoInput(target, text);
                break;
            case 'code':
                this.insertIntoCodeEditor(target, text);
                break;
            default:
                this.insertIntoInput(target, text);
        }

        // Trigger input events for frameworks like React
        this.triggerInputEvents(target);
    }

    insertIntoInput(element, text) {
        const start = element.selectionStart || element.value.length;
        const end = element.selectionEnd || element.value.length;
        const beforeText = element.value.substring(0, start);
        const afterText = element.value.substring(end);

        element.value = beforeText + text + afterText;
        element.selectionStart = element.selectionEnd = start + text.length;
    }

    insertIntoContentEditable(element, text) {
        const selection = window.getSelection();
        if (selection.rangeCount > 0) {
            const range = selection.getRangeAt(0);
            range.deleteContents();
            const textNode = document.createTextNode(text);
            range.insertNode(textNode);
            range.setStartAfter(textNode);
            range.setEndAfter(textNode);
            selection.removeAllRanges();
            selection.addRange(range);
        } else {
            // Fallback: append to end
            element.textContent += text;
        }
    }

    insertIntoCodeEditor(element, text) {
        // Handle specific code editors
        if (element.classList.contains('monaco-editor')) {
            // VS Code style editor
            return this.insertIntoMonacoEditor(element, text);
        }

        // Generic code insertion
        this.insertIntoInput(element, text);
    }

    insertIntoMonacoEditor(element, text) {
        // Monaco editor integration (used by VS Code, etc.)
        const editor = element.closest('.monaco-editor');
        if (editor && window.monaco) {
            // This would need actual Monaco API integration
            console.log('Monaco editor detected, text insertion would need API integration');
            this.insertIntoInput(element, text);
        } else {
            this.insertIntoInput(element, text);
        }
    }

    triggerInputEvents(element) {
        // Trigger events that frameworks listen for
        const events = ['input', 'change', 'keydown', 'keyup'];

        events.forEach(eventType => {
            const event = new Event(eventType, { bubbles: true });
            element.dispatchEvent(event);
        });

        // React specific
        if (element._valueTracker) {
            element._valueTracker.setValue('');
        }
    }

    isInputElement(element) {
        if (!element) return false;

        const tagName = element.tagName.toLowerCase();
        const inputTypes = ['input', 'textarea', 'select'];

        return inputTypes.includes(tagName) ||
               element.contentEditable === 'true' ||
               element.getAttribute('role') === 'textbox' ||
               element.classList.contains('ace_editor') ||
               element.classList.contains('CodeMirror');
    }

    getElementInfo(element) {
        if (!element) return null;

        return {
            tagName: element.tagName,
            type: element.type,
            className: element.className,
            id: element.id,
            name: element.name,
            contentEditable: element.contentEditable,
            role: element.getAttribute('role'),
            placeholder: element.placeholder
        };
    }

    showDictationIndicator(element) {
        if (!element) return;

        // Remove existing indicator
        this.hideDictationIndicator();

        // Create indicator
        const indicator = document.createElement('div');
        indicator.id = 'vibespeak-dictation-indicator';
        indicator.innerHTML = `
            <div style="
                position: absolute;
                background: #007bff;
                color: white;
                padding: 4px 8px;
                border-radius: 4px;
                font-size: 12px;
                z-index: 10000;
                pointer-events: none;
                box-shadow: 0 2px 4px rgba(0,0,0,0.2);
            ">
                🎤 Dictation Ready (Ctrl+Shift+D)
            </div>
        `;

        document.body.appendChild(indicator);

        // Position near the input
        const rect = element.getBoundingClientRect();
        const indicatorEl = indicator.firstElementChild;
        indicatorEl.style.left = rect.left + 'px';
        indicatorEl.style.top = (rect.top - 30) + 'px';
    }

    hideDictationIndicator() {
        const indicator = document.getElementById('vibespeak-dictation-indicator');
        if (indicator) {
            indicator.remove();
        }
    }

    showDictationActiveIndicator() {
        const indicator = document.createElement('div');
        indicator.id = 'vibespeak-dictation-active';
        indicator.innerHTML = `
            <div style="
                position: fixed;
                top: 20px;
                right: 20px;
                background: #28a745;
                color: white;
                padding: 8px 12px;
                border-radius: 6px;
                font-size: 14px;
                z-index: 10001;
                box-shadow: 0 2px 8px rgba(0,0,0,0.3);
                animation: pulse 2s infinite;
            ">
                🎤 Dictating...
            </div>
            <style>
                @keyframes pulse {
                    0% { opacity: 1; }
                    50% { opacity: 0.7; }
                    100% { opacity: 1; }
                }
            </style>
        `;

        document.body.appendChild(indicator);
    }

    hideDictationActiveIndicator() {
        const indicator = document.getElementById('vibespeak-dictation-active');
        if (indicator) {
            indicator.remove();
        }
    }

    notifyExtension(message) {
        chrome.runtime.sendMessage(message);
    }

    toggleDictation() {
        if (this.isDictationActive) {
            this.stopDictation();
        } else {
            const inputType = this.currentInput ? this.detectInputType(this.currentInput) : 'text';
            this.startDictation(inputType);
        }
    }

    detectInputType(element) {
        if (element.contentEditable === 'true') return 'contenteditable';
        if (element.tagName === 'TEXTAREA') return 'textarea';
        if (element.classList.contains('ace_editor') || element.classList.contains('CodeMirror')) return 'code';
        return 'input';
    }
}

// Initialize dictation manager when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        new DictationManager();
    });
} else {
    new DictationManager();
}