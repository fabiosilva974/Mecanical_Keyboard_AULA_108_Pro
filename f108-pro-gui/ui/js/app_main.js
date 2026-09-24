/**
 * app_main.js - Main Application Orchestrator for AULA F108 Pro GUI.
 */

class AppMain {
    constructor() {
        this.canvas = null;
        this.composer = null;
        this.activeTab = 'home';
        
        // Tab Controllers
        this.tabRemap = null;
        this.tabLighting = null;
        this.tabPerKey = null;
        this.tabLcd = null;
        this.tabSettings = null;

        // 20 Official Lighting Modes
        this.lightingModes = [
            'Off', 'Static', 'SingleOn', 'SingleOff', 'Glittering',
            'Falling', 'Colourful', 'Breath', 'Spectrum', 'Outward',
            'Scrolling', 'Rolling', 'Rotating', 'Explode', 'Launch',
            'Ripples', 'Flowing', 'Pulsating', 'Tilt', 'Shuttle'
        ];

        this.presetColors = [
            '#9333EA', '#3B82F6', '#06B6D4', '#10B981', '#EAB308',
            '#F97316', '#EF4444', '#EC4899', '#FFFFFF', '#000000'
        ];
    }

    async init() {
        console.log('[APP] Initializing AULA F108 Pro Suite...');

        // 1. Initialize i18n
        await window.i18n.init();

        // 2. Initialize CLI Composer
        this.composer = new window.CliComposer();
        this.composer.init();

        // 3. Initialize Keyboard Canvas
        this.canvas = new window.KeyboardCanvas('keyboard-stage-container');
        await this.canvas.init();

        // 4. Instantiate and initialize Tab Controllers
        this.tabRemap = new window.TabRemap(this);
        this.tabRemap.init();

        this.tabLighting = new window.TabLighting(this);
        this.tabLighting.init();

        this.tabPerKey = new window.TabPerKey(this);
        this.tabPerKey.init();

        this.tabLcd = new window.TabLcd(this);
        this.tabLcd.init();

        this.tabSettings = new window.TabSettings(this);
        this.tabSettings.init();

        // Canvas events
        this.canvas.onKeySelected((keyObj) => this.onKeySelected(keyObj));
        this.canvas.onScreenClicked(() => this.switchTab('lcd'));

        // 5. Populate Dynamic Lighting controls
        this.populateLightingModes();
        this.populateLightingSwatches();

        // 6. Setup Navigation & Preferences
        this.setupNavigation();
        this.setupPreferences();
        this.setupActionListeners();

        // 7. Connect to Hardware USB Status
        this.connectHardware();

        console.log('[APP] Ready!');
    }

    setupNavigation() {
        document.querySelectorAll('.nav-item').forEach(btn => {
            btn.addEventListener('click', () => {
                this.switchTab(btn.dataset.tab);
            });
        });
    }

    switchTab(tabId) {
        this.activeTab = tabId;

        document.querySelectorAll('.nav-item').forEach(b => {
            b.classList.toggle('active', b.dataset.tab === tabId);
        });

        document.querySelectorAll('.tab-pane').forEach(p => {
            p.classList.toggle('active', p.id === `pane-${tabId}`);
        });

        if (tabId === 'lighting') {
            this.tabLighting.updateCommand();
        } else if (tabId === 'lcd') {
            this.composer.setCommand('f108-pro clock');
        } else if (tabId === 'home') {
            this.composer.setCommand('f108-pro --help');
        } else if (tabId === 'remap') {
            this.tabRemap.updateCommand();
        } else if (tabId === 'perkey') {
            this.tabPerKey.updateCommand();
        }
    }

    onKeySelected(keyObj) {
        if (this.activeTab === 'remap') {
            this.tabRemap.onKeySelected(keyObj);
        } else if (this.activeTab === 'perkey') {
            if (keyObj) {
                const color = document.getElementById('perkey-color-picker').value;
                this.tabPerKey.paintKey(keyObj.id, color);
            }
        }
    }

    populateLightingModes() {
        const select = document.getElementById('lighting-mode-select');
        select.innerHTML = '';
        this.lightingModes.forEach(mode => {
            const opt = document.createElement('option');
            opt.value = mode;
            opt.textContent = `${mode} - ${window.i18n.t(`lighting.modes.${mode}`, mode)}`;
            select.appendChild(opt);
        });
        select.value = 'Breath';

        select.addEventListener('change', () => this.tabLighting.updateCommand());

        const bSlider = document.getElementById('slider-brightness');
        const bVal = document.getElementById('val-brightness');
        bSlider.addEventListener('input', (e) => {
            bVal.textContent = e.target.value;
            this.tabLighting.updateCommand();
        });

        const sSlider = document.getElementById('slider-speed');
        const sVal = document.getElementById('val-speed');
        sSlider.addEventListener('input', (e) => {
            sVal.textContent = e.target.value;
            this.tabLighting.updateCommand();
        });

        const colorPicker = document.getElementById('lighting-color-picker');
        const colorHex = document.getElementById('lighting-color-hex');
        colorPicker.addEventListener('input', (e) => {
            colorHex.value = e.target.value.toUpperCase();
            this.tabLighting.updateCommand();
        });
        colorHex.addEventListener('change', (e) => {
            colorPicker.value = e.target.value;
            this.tabLighting.updateCommand();
        });
    }

    populateLightingSwatches() {
        const container = document.getElementById('lighting-color-swatches');
        container.innerHTML = '';
        this.presetColors.forEach(c => {
            const swatch = document.createElement('div');
            swatch.style.width = '24px';
            swatch.style.height = '24px';
            swatch.style.borderRadius = '4px';
            swatch.style.background = c;
            swatch.style.cursor = 'pointer';
            swatch.style.border = '1px solid rgba(255,255,255,0.2)';
            swatch.addEventListener('click', () => {
                document.getElementById('lighting-color-picker').value = c;
                document.getElementById('lighting-color-hex').value = c.toUpperCase();
                this.tabLighting.updateCommand();
            });
            container.appendChild(swatch);
        });
    }

    setupPreferences() {
        const themeSelect = document.getElementById('select-theme');
        const savedTheme = localStorage.getItem('app_theme') || 'theme-purple';
        document.body.className = savedTheme;
        themeSelect.value = savedTheme;

        themeSelect.addEventListener('change', (e) => {
            const theme = e.target.value;
            document.body.className = theme;
            localStorage.setItem('app_theme', theme);
        });

        const langSelectTop = document.getElementById('select-lang');
        const langSelectSettings = document.getElementById('select-settings-lang');
        langSelectTop.value = window.i18n.currentLocale;
        langSelectSettings.value = window.i18n.currentLocale;

        const onLangChange = async (e) => {
            const loc = e.target.value;
            langSelectTop.value = loc;
            langSelectSettings.value = loc;
            await window.i18n.setLocale(loc);
            this.populateLightingModes();
        };

        langSelectTop.addEventListener('change', onLangChange);
        langSelectSettings.addEventListener('change', onLangChange);
    }

    setupActionListeners() {
        const syncClock = () => {
            this.composer.setCommand('f108-pro clock');
            this.composer.runCurrentCommand();
        };
        document.getElementById('btn-quick-sync-clock').addEventListener('click', syncClock);
        document.getElementById('btn-home-sync-clock').addEventListener('click', syncClock);

        const lightsOff = () => {
            this.composer.setCommand('f108-pro off');
            this.composer.runCurrentCommand();
        };
        document.getElementById('btn-quick-lights-off').addEventListener('click', lightsOff);
        document.getElementById('btn-home-lights-off').addEventListener('click', lightsOff);

        document.getElementById('btn-home-lights-default').addEventListener('click', () => {
            this.composer.setCommand('f108-pro light Breath 5 3 147 51 234');
            this.composer.runCurrentCommand();
        });
    }

    async connectHardware() {
        const dot = document.getElementById('status-dot');
        const text = document.getElementById('status-text');
        const detail = document.getElementById('home-conn-detail');

        const updateStatus = (status) => {
            if (status.cable_connected) {
                dot.className = 'status-dot connected';
                text.textContent = 'USB Wired (Full Access)';
                detail.textContent = 'Connected via USB Cable (0c45:800a) - Full feature reports ready';
                detail.style.color = 'var(--success)';
            } else if (status.dongle_connected) {
                dot.className = 'status-dot';
                text.textContent = '2.4G Wireless (Connect USB for Settings)';
                detail.textContent = 'Connected via 2.4G Dongle (05ac:024f) - Connect USB cable to configure hardware';
                detail.style.color = 'var(--warning)';
            } else {
                dot.className = 'status-dot disconnected';
                text.textContent = 'Disconnected';
                detail.textContent = 'No keyboard detected on USB';
                detail.style.color = 'var(--error)';
            }
        };

        window.hostBridge.on('usb_status_changed', (status) => updateStatus(status));

        try {
            const status = await window.hostBridge.invoke('get_device_status');
            updateStatus(status);
        } catch (e) {
            console.error('Initial status query error:', e);
        }
    }

    hexToRgb(hex) {
        let c = hex.replace('#', '');
        if (c.length === 3) c = c.split('').map(x => x + x).join('');
        const num = parseInt(c, 16);
        return {
            r: (num >> 16) & 255,
            g: (num >> 8) & 255,
            b: num & 255
        };
    }
}

window.addEventListener('DOMContentLoaded', () => {
    window.app = new AppMain();
    window.app.init();
});
