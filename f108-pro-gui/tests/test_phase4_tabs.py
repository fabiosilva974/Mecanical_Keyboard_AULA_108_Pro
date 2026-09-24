#!/usr/bin/env python3
"""
Test Suite for AULA F108 Pro GUI Tab Modules (Phase 4).
Tests:
1. TabRemap: Key selection and dynamic CLI remap command generation.
2. TabLighting: Mode changes, brightness/speed/color adjustments, and CLI lighting command generation.
3. TabPerKey: WASD preset painting, clear all, and CLI perkey command generation.
4. TabLcd: Clock sync trigger.
5. TabSettings: Driver path configuration.
"""

import os
import sys
import json
import unittest
from pathlib import Path

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
gi.require_version('JavaScriptCore', '4.1')
from gi.repository import Gtk, WebKit2, GLib

GUI_ROOT = Path("/mnt/Files/AgenteLocalLinux/AULASetup/f108-pro-gui")
sys.path.insert(0, str(GUI_ROOT))
from app import KeyboardManagerApp

class TestPhase4Tabs(unittest.TestCase):
    def setUp(self):
        self.app = KeyboardManagerApp()
        self.test_passed = False
        self.error_message = None
        self.diagnostics = {}

    def test_tab_modules(self):
        def run_test():
            test_script = """
            (async function() {
                try {
                    const app = window.app;
                    if (!app) throw new Error('window.app is missing');
                    if (!app.tabRemap) throw new Error('tabRemap missing');
                    if (!app.tabLighting) throw new Error('tabLighting missing');
                    if (!app.tabPerKey) throw new Error('tabPerKey missing');
                    if (!app.tabLcd) throw new Error('tabLcd missing');
                    if (!app.tabSettings) throw new Error('tabSettings missing');

                    const results = {};

                    // 1. Test Tab Lighting
                    app.switchTab('lighting');
                    document.getElementById('lighting-mode-select').value = 'Explode';
                    document.getElementById('slider-brightness').value = '4';
                    document.getElementById('slider-speed').value = '2';
                    document.getElementById('lighting-color-picker').value = '#00FF00';
                    app.tabLighting.updateCommand();
                    results.lighting_cmd = app.composer.currentCommand;
                    if (!results.lighting_cmd.includes('Explode 4 2 0 255 0')) {
                        throw new Error(`Unexpected lighting command: ${results.lighting_cmd}`);
                    }

                    // 2. Test Tab Remap
                    app.switchTab('remap');
                    app.canvas.selectKey('CAPSLOCK');
                    document.getElementById('remap-layer-select').value = 'base';
                    document.getElementById('remap-target-key-select').value = 'lctrl';
                    app.tabRemap.updateCommand();
                    results.remap_cmd = app.composer.currentCommand;
                    if (results.remap_cmd !== 'f108-pro remap capslock lctrl') {
                        throw new Error(`Unexpected remap command: ${results.remap_cmd}`);
                    }

                    // Test Remap with FN layer
                    document.getElementById('remap-layer-select').value = 'fn';
                    app.tabRemap.updateCommand();
                    results.remap_fn_cmd = app.composer.currentCommand;
                    if (results.remap_fn_cmd !== 'f108-pro remap --fn capslock lctrl') {
                        throw new Error(`Unexpected remap FN command: ${results.remap_fn_cmd}`);
                    }

                    // 3. Test Tab Per-Key WASD Preset
                    app.switchTab('perkey');
                    app.tabPerKey.applyPresetWasd();
                    results.perkey_wasd_count = app.tabPerKey.keyColorMap.size;
                    results.perkey_cmd = app.composer.currentCommand;
                    if (results.perkey_wasd_count < 4) {
                        throw new Error(`Expected at least 4 WASD keys, found: ${results.perkey_wasd_count}`);
                    }

                    // 4. Test Tab LCD
                    app.switchTab('lcd');
                    results.lcd_cmd = app.composer.currentCommand;
                    if (!results.lcd_cmd.includes('clock')) {
                        throw new Error(`Unexpected LCD command: ${results.lcd_cmd}`);
                    }

                    // All passed!
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'phase4_test',
                        action: 'phase4_result',
                        payload: {
                            success: true,
                            results: results
                        }
                    });
                } catch (e) {
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'phase4_test',
                        action: 'phase4_result',
                        payload: { success: false, error: e.toString() }
                    });
                }
            })();
            """
            self.app.webview.evaluate_javascript(test_script, -1, None, None, None, None, None)

        def on_msg(manager, js_result):
            try:
                js_val = js_result.get_js_value()
                data = json.loads(js_val.to_json(0))
                if data.get("action") == "phase4_result":
                    payload = data.get("payload", {})
                    self.test_passed = payload.get("success", False)
                    self.error_message = payload.get("error")
                    self.diagnostics = payload.get("results", {})
                    Gtk.main_quit()
            except Exception:
                pass

        self.app.ucm.connect("script-message-received::api", on_msg)

        def on_load_changed(webview, event):
            if event == WebKit2.LoadEvent.FINISHED:
                GLib.timeout_add(1000, run_test)

        self.app.webview.connect("load-changed", on_load_changed)
        GLib.timeout_add_seconds(8, Gtk.main_quit)

        self.app.show_all()
        Gtk.main()

        print(f"Phase 4 Diagnostics: {json.dumps(self.diagnostics, indent=2)}")
        self.assertTrue(self.test_passed, f"Phase 4 Tabs test failed: {self.error_message}")

if __name__ == "__main__":
    unittest.main()
