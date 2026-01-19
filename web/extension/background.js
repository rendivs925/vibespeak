// Vibespeak Dictation Background Script
// Handles communication between content scripts and the dictation server

class DictationBackground {
    constructor() {
        this.dictationServer = 'http://localhost:8080';
        this.activeTabs = new Map();

        this.init();
    }

    init() {
        // Handle messages from content scripts
        chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
            this.handleMessage(request, sender, sendResponse);
            return true;
        });

        // Handle extension icon clicks
        chrome.action.onClicked.addListener((tab) => {
            this.toggleDictationForTab(tab.id);
        });

        // Handle tab updates
        chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
            if (changeInfo.status === 'complete') {
                this.notifyTabOfDictationStatus(tabId);
            }
        });

        // Handle tab removal
        chrome.tabs.onRemoved.addListener((tabId) => {
            this.activeTabs.delete(tabId);
        });
    }

    async handleMessage(request, sender, sendResponse) {
        const tabId = sender.tab?.id;

        switch (request.action) {
            case 'dictationStarted':
                this.activeTabs.set(tabId, { active: true, inputType: request.inputType });
                this.updateExtensionIcon(tabId, true);
                break;

            case 'dictationStopped':
                this.activeTabs.set(tabId, { active: false });
                this.updateExtensionIcon(tabId, false);
                break;

            case 'startDictation':
                await this.startDictationForTab(tabId, request.inputType);
                break;

            case 'stopDictation':
                await this.stopDictationForTab(tabId);
                break;

            case 'insertText':
                await this.insertTextInTab(tabId, request.text, request.inputType);
                break;
        }
    }

    async startDictationForTab(tabId, inputType) {
        try {
            await chrome.scripting.executeScript({
                target: { tabId },
                function: (inputType) => {
                    window.vibespeakDictation?.startDictation(inputType);
                },
                args: [inputType]
            });

            this.activeTabs.set(tabId, { active: true, inputType });
            this.updateExtensionIcon(tabId, true);
        } catch (error) {
            console.error('Failed to start dictation:', error);
        }
    }

    async stopDictationForTab(tabId) {
        try {
            await chrome.scripting.executeScript({
                target: { tabId },
                function: () => {
                    window.vibespeakDictation?.stopDictation();
                }
            });

            this.activeTabs.set(tabId, { active: false });
            this.updateExtensionIcon(tabId, false);
        } catch (error) {
            console.error('Failed to stop dictation:', error);
        }
    }

    async insertTextInTab(tabId, text, inputType) {
        try {
            await chrome.scripting.executeScript({
                target: { tabId },
                function: (text, inputType) => {
                    window.vibespeakDictation?.insertText(text, inputType);
                },
                args: [text, inputType]
            });
        } catch (error) {
            console.error('Failed to insert text:', error);
        }
    }

    async toggleDictationForTab(tabId) {
        const tabStatus = this.activeTabs.get(tabId);

        if (tabStatus?.active) {
            await this.stopDictationForTab(tabId);
        } else {
            // Get the active element info first
            try {
                const results = await chrome.scripting.executeScript({
                    target: { tabId },
                    function: () => {
                        return window.vibespeakDictation?.getActiveElement?.() || {
                            isInput: document.activeElement?.tagName === 'INPUT' ||
                                   document.activeElement?.tagName === 'TEXTAREA' ||
                                   document.activeElement?.contentEditable === 'true'
                        };
                    }
                });

                const activeElement = results[0]?.result;
                if (activeElement?.isInput) {
                    const inputType = this.detectInputType(activeElement.element);
                    await this.startDictationForTab(tabId, inputType);
                } else {
                    // Show notification if no suitable input found
                    chrome.notifications.create({
                        type: 'basic',
                        iconUrl: 'icon128.png',
                        title: 'Vibespeak Dictation',
                        message: 'Please focus on a text input field first'
                    });
                }
            } catch (error) {
                console.error('Failed to check active element:', error);
            }
        }
    }

    detectInputType(elementInfo) {
        if (!elementInfo) return 'text';

        if (elementInfo.contentEditable === 'true') return 'contenteditable';
        if (elementInfo.tagName === 'TEXTAREA') return 'textarea';
        if (elementInfo.className?.includes('monaco-editor') ||
            elementInfo.className?.includes('ace_editor') ||
            elementInfo.className?.includes('CodeMirror')) return 'code';

        return 'input';
    }

    updateExtensionIcon(tabId, isActive) {
        const iconPath = isActive ? 'icon-active.png' : 'icon128.png';

        chrome.action.setIcon({
            tabId,
            path: iconPath
        });

        chrome.action.setBadgeText({
            tabId,
            text: isActive ? 'ON' : ''
        });

        chrome.action.setBadgeBackgroundColor({
            tabId,
            color: isActive ? '#28a745' : '#666'
        });
    }

    notifyTabOfDictationStatus(tabId) {
        const status = this.activeTabs.get(tabId);
        if (status) {
            chrome.tabs.sendMessage(tabId, {
                action: 'dictationStatusUpdate',
                active: status.active,
                inputType: status.inputType
            });
        }
    }
}

// Initialize background script
new DictationBackground();