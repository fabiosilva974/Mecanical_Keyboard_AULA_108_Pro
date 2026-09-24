/**
 * tab_lighting.js - Global RGB Lighting Effects Controller.
 */

class TabLighting {
    constructor(app) {
        this.app = app;
    }

    init() {
        this.modeSelect = document.getElementById('lighting-mode-select');
        this.sliderBrightness = document.getElementById('slider-brightness');
        this.valBrightness = document.getElementById('val-brightness');
        this.sliderSpeed = document.getElementById('slider-speed');
        this.valSpeed = document.getElementById('val-speed');
        this.colorPicker = document.getElementById('lighting-color-picker');
        this.colorHex = document.getElementById('lighting-color-hex');
        this.applyBtn = document.getElementById('btn-apply-lighting');

        this.applyBtn.addEventListener('click', () => {
            this.updateCommand();
            this.app.composer.runCurrentCommand();
        });
    }

    updateCommand() {
        const mode = this.modeSelect.value;
        const b = this.sliderBrightness.value;
        const s = this.sliderSpeed.value;
        const hex = this.colorPicker.value;
        const rgb = this.app.hexToRgb(hex);

        const cmd = `f108-pro light ${mode} ${b} ${s} ${rgb.r} ${rgb.g} ${rgb.b}`;
        this.app.composer.setCommand(cmd);
    }
}

window.TabLighting = TabLighting;
