// Vibespeak Dictation Popup Script

class DictationPopup {
    constructor() {
        this.dictationActive = false;
        this.currentTab = null;

        this.init();
    }

    async init() {
        // Get current tab
        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        this.currentTab = tab;

        // Check if dictation is active for this tab
        this.updateStatus();

        // Set up event listeners
        document.getElementById('toggle-dictation').addEventListener('click', () => {
            this.toggleDictation();
        });

        document.getElementById('stop-dictation').addEventListener('click', () => {
            this.stopDictation();
        });

        // Listen for status updates
        chrome.runtime.onMessage.addListener((request) => {
            if (request.action === 'dictationStatusUpdate') {
                this.updateStatusFromMessage(request);
            }
        });
    }

    async updateStatus() {
        try {
            // Check if content script is available and get dictation status
            const results = await chrome.scripting.executeScript({
                target: { tabId: this.currentTab.id },
                function: () => {
                    return {
                        isActive: window.vibespeakDictation?.isDictationActive || false,
                        currentInput: window.vibespeakDictation?.currentInput ? {
                            tagName: window.vibespeakDictation.currentInput.tagName,
                            type: window.vibespeakDictation.currentInput.type,
                            className: window.vibespeakDictation.currentInput.className
                        } : null
                    };
                }
            });

            const status = results[0]?.result;
            this.updateUI(status?.isActive || false, status?.currentInput);
        } catch (error) {
            console.error('Failed to get dictation status:', error);
            this.updateUI(false);
        }
    }

    updateUI(isActive, currentInput = null) {
        this.dictationActive = isActive;
        const statusEl = document.getElementById('status');
        const toggleBtn = document.getElementById('toggle-dictation');
        const stopBtn = document.getElementById('stop-dictation');

        if (isActive) {
            statusEl.className = 'status active';
            statusEl.textContent = currentInput ?
                `Dictating in ${currentInput.tagName.toLowerCase()}` :
                'Dictation active';

            toggleBtn.style.display = 'none';
            stopBtn.style.display = 'block';
        } else {
            statusEl.className = 'status inactive';
            statusEl.textContent = 'Dictation inactive';

            toggleBtn.style.display = 'block';
            stopBtn.style.display = 'none';
        }
    }

    updateStatusFromMessage(message) {
        this.updateUI(message.active, null);
    }

    async toggleDictation() {
        if (this.dictationActive) {
            await this.stopDictation();
        } else {
            await this.startDictation();
        }
    }

    async startDictation() {
        try {
            // Get active element info
            const results = await chrome.scripting.executeScript({
                target: { tabId: this.currentTab.id },
                function: () => {
                    const element = document.activeElement;
                    if (!element) return null;

                    return {
                        tagName: element.tagName,
                        type: element.type,
                        className: element.className,
                        contentEditable: element.contentEditable,
                        role: element.getAttribute('role')
                    };
                }
            });

            const activeElement = results[0]?.result;
            let inputType = 'text';

            if (activeElement) {
                if (activeElement.contentEditable === 'true') {
                    inputType = 'contenteditable';
                } else if (activeElement.tagName === 'TEXTAREA') {
                    inputType = 'textarea';
                } else if (activeElement.className?.includes('monaco-editor') ||
                          activeElement.className?.includes('ace_editor') ||
                          activeElement.className?.includes('CodeMirror')) {
                    inputType = 'code';
                }
            }

            // Start dictation via background script
            chrome.runtime.sendMessage({
                action: 'startDictation',
                inputType: inputType
            });

            this.updateUI(true, activeElement);
        } catch (error) {
            console.error('Failed to start dictation:', error);
        }
    }

    async stopDictation() {
        try {
            chrome.runtime.sendMessage({
                action: 'stopDictation'
            });

            this.updateUI(false);
        } catch (error) {
            console.error('Failed to stop dictation:', error);
        }
    }
}

// Initialize popup when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    new DictationPopup();
});