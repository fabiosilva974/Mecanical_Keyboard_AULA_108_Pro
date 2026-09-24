/**
 * tab_remap.js - Physical Key Remapping Module.
 */

class TabRemap {
    constructor(app) {
        this.app = app;
        this.layer = 'base'; // 'base' or 'fn'
        this.actionType = 'key'; // 'key', 'combo', 'consumer', 'mouse', 'disabled'
        this.selectedKey = null;
        this.remapRules = new Map(); // keyId -> rule

        this.standardKeys = [
            'ESC', 'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12',
            'PRINT', 'SCROLL', 'PAUSE', 'GRAVE', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
            'MINUS', 'EQUALS', 'BACKSPACE', 'INSERT', 'HOME', 'PAGEUP', 'NUMLOCK', 'KP_DIVIDE',
            'KP_MULTIPLY', 'KP_MINUS', 'TAB', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P',
            'LBRACKET', 'RBRACKET', 'BACKSLASH', 'DELETE', 'END', 'PAGEDOWN', 'KP_7', 'KP_8', 'KP_9',
            'KP_PLUS', 'CAPSLOCK', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'SEMICOLON', 'QUOTE',
            'ENTER', 'KP_4', 'KP_5', 'KP_6', 'LSHIFT', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', 'COMMA',
            'DOT', 'SLASH', 'RSHIFT', 'UP', 'KP_1', 'KP_2', 'KP_3', 'KP_ENTER', 'LCTRL', 'LWIN',
            'LALT', 'SPACE', 'RALT', 'RWIN', 'FN', 'RCTRL', 'LEFT', 'DOWN', 'RIGHT', 'KP_0', 'KP_DOT'
        ];

        this.consumerActions = [
            { id: 'vol_up', name: 'Volume Up' },
            { id: 'vol_down', name: 'Volume Down' },
            { id: 'mute', name: 'Mute Audio' },
            { id: 'play_pause', name: 'Play / Pause' },
            { id: 'next', name: 'Next Track' },
            { id: 'prev', name: 'Previous Track' },
            { id: 'calc', name: 'Calculator' },
            { id: 'browser', name: 'Web Browser' },
            { id: 'email', name: 'Email Client' }
        ];

        this.mouseActions = [
            { id: 'left_click', name: 'Mouse Left Click' },
            { id: 'right_click', name: 'Mouse Right Click' },
            { id: 'middle_click', name: 'Mouse Middle Click' },
            { id: 'wheel_up', name: 'Mouse Scroll Up' },
            { id: 'wheel_down', name: 'Mouse Scroll Down' }
        ];
    }

    init() {
        this.layerSelect = document.getElementById('remap-layer-select');
        this.actionTypeSelect = document.getElementById('remap-action-type');
        this.targetKeySelect = document.getElementById('remap-target-key-select');
        this.keyNameInput = document.getElementById('remap-selected-key-name');
        this.applyBtn = document.getElementById('btn-apply-remap');
        this.resetKeyBtn = document.getElementById('btn-reset-key');
        this.exportBtn = document.getElementById('btn-export-yaml');
        this.importBtn = document.getElementById('btn-import-yaml');

        this.populateTargetKeys();

        this.layerSelect.addEventListener('change', (e) => {
            this.layer = e.target.value;
            this.updateCommand();
        });

        this.actionTypeSelect.addEventListener('change', (e) => {
            this.actionType = e.target.value;
            this.populateTargetKeys();
            this.updateCommand();
        });

        this.targetKeySelect.addEventListener('change', () => this.updateCommand());

        this.applyBtn.addEventListener('click', () => this.applyRemap());
        this.resetKeyBtn.addEventListener('click', () => this.resetKey());
        this.exportBtn.addEventListener('click', () => this.exportYaml());
        this.importBtn.addEventListener('click', () => this.importYaml());
    }

    populateTargetKeys() {
        this.targetKeySelect.innerHTML = '';
        if (this.actionType === 'key' || this.actionType === 'combo') {
            this.standardKeys.forEach(k => {
                const opt = document.createElement('option');
                opt.value = k.toLowerCase();
                opt.textContent = k;
                this.targetKeySelect.appendChild(opt);
            });
            this.targetKeySelect.value = 'lctrl';
        } else if (this.actionType === 'consumer') {
            this.consumerActions.forEach(a => {
                const opt = document.createElement('option');
                opt.value = a.id;
                opt.textContent = a.name;
                this.targetKeySelect.appendChild(opt);
            });
        } else if (this.actionType === 'mouse') {
            this.mouseActions.forEach(m => {
                const opt = document.createElement('option');
                opt.value = m.id;
                opt.textContent = m.name;
                this.targetKeySelect.appendChild(opt);
            });
        } else if (this.actionType === 'disabled') {
            const opt = document.createElement('option');
            opt.value = 'none';
            opt.textContent = 'None / Disabled';
            this.targetKeySelect.appendChild(opt);
        }
    }

    onKeySelected(keyObj) {
        this.selectedKey = keyObj ? keyObj.id : null;
        if (keyObj) {
            this.keyNameInput.value = `${keyObj.label} (${keyObj.id})`;
        } else {
            this.keyNameInput.value = '';
        }
        this.updateCommand();
    }

    updateCommand() {
        if (!this.selectedKey) {
            this.app.composer.setCommand('f108-pro remap --help');
            return;
        }

        this.layer = this.layerSelect ? this.layerSelect.value : this.layer;
        this.actionType = this.actionTypeSelect ? this.actionTypeSelect.value : this.actionType;

        const fnFlag = this.layer === 'fn' ? '--fn ' : '';
        const src = this.selectedKey.toLowerCase();
        let cmd = '';

        if (this.actionType === 'key') {
            const tgt = this.targetKeySelect.value;
            cmd = `f108-pro remap ${fnFlag}${src} ${tgt}`;
        } else if (this.actionType === 'combo') {
            const tgt = this.targetKeySelect.value;
            cmd = `f108-pro remap ${fnFlag}${src} ${tgt} --ctrl`;
        } else if (this.actionType === 'consumer') {
            const act = this.targetKeySelect.value;
            cmd = `f108-pro remap ${fnFlag}${src} --consumer ${act}`;
        } else if (this.actionType === 'mouse') {
            const act = this.targetKeySelect.value;
            cmd = `f108-pro remap ${fnFlag}${src} --mouse ${act}`;
        } else if (this.actionType === 'disabled') {
            cmd = `f108-pro remap ${fnFlag}${src} none`;
        }

        this.app.composer.setCommand(cmd);
    }

    async applyRemap() {
        if (!this.selectedKey) return;
        this.updateCommand();
        await this.app.composer.runCurrentCommand();
        this.app.canvas.setKeyRemapped(this.selectedKey, true);
    }

    async resetKey() {
        if (!this.selectedKey) return;
        const fnFlag = this.layer === 'fn' ? '--fn ' : '';
        const src = this.selectedKey.toLowerCase();
        this.app.composer.setCommand(`f108-pro remap ${fnFlag}${src} ${src}`);
        await this.app.composer.runCurrentCommand();
        this.app.canvas.setKeyRemapped(this.selectedKey, false);
    }

    async exportYaml() {
        try {
            const fileRes = await window.hostBridge.invoke('save_file_dialog', {
                title: 'Export Key Remap Profile',
                default_name: 'f108_remap_profile.yaml'
            });
            if (fileRes && fileRes.path) {
                const yamlContent = "# AULA F108 Pro Key Remap Profile\nlayer: " + this.layer + "\nmappings:\n  capslock: lctrl\n";
                await window.hostBridge.invoke('write_file', {
                    path: fileRes.path,
                    content: yamlContent
                });
                this.app.composer.appendLog(`[OK] Remap profile exported to: ${fileRes.path}\n`, 'info');
            }
        } catch (e) {
            console.error('Export failed:', e);
        }
    }

    async importYaml() {
        try {
            const fileRes = await window.hostBridge.invoke('open_file_dialog', {
                title: 'Import Key Remap Profile',
                filter: 'yaml'
            });
            if (fileRes && fileRes.path) {
                this.app.composer.setCommand(`f108-pro remap --file ${fileRes.path}`);
                this.app.composer.appendLog(`[INFO] Loaded remap profile: ${fileRes.path}\n`, 'info');
            }
        } catch (e) {
            console.error('Import failed:', e);
        }
    }
}

window.TabRemap = TabRemap;
