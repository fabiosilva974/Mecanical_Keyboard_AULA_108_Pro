#!/usr/bin/env python3
"""
Test Suite for AULA F108 Pro GUI (Phase 3).
Validates UI loading, Canvas rendering (104 keys), CLI composer initialization, and i18n engine.
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

class TestPhase3UI(unittest.TestCase):
    def setUp(self):
        self.app = KeyboardManagerApp()
        self.test_passed = False
        self.error_message = None
        self.diagnostics = {}

    def test_phase3_components(self):
        def run_test():
            test_script = """
            (async function() {
                try {
                    // Check app
                    if (!window.app) throw new Error('window.app is not defined');
                    if (!window.i18n) throw new Error('window.i18n is not defined');
                    if (!window.app.canvas) throw new Error('window.app.canvas is not defined');
                    if (!window.app.composer) throw new Error('window.app.composer is not defined');

                    // Check rendered keys
                    const keycaps = document.querySelectorAll('.keycap');
                    if (keycaps.length !== 104) {
                        throw new Error(`Expected 104 keycaps rendered, found: ${keycaps.length}`);
                    }

                    // Check composer preview element
                    const preview = document.getElementById('cli-cmd-preview');
                    if (!preview || !preview.textContent.includes('f108-pro')) {
                        throw new Error('CLI composer preview element missing or invalid');
                    }

                    // Check active theme
                    const bodyTheme = document.body.className;
                    if (!bodyTheme) throw new Error('No active theme class on body');

                    // Check i18n translation
                    const homeTabTitle = document.querySelector('[data-i18n="nav.home"]');
                    if (!homeTabTitle || homeTabTitle.textContent !== 'Home') {
                        throw new Error(`i18n failed to translate nav.home, text is: '${homeTabTitle ? homeTabTitle.textContent : "null"}'`);
                    }

                    // Return diagnostic report
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'phase3_test',
                        action: 'phase3_result',
                        payload: {
                            success: true,
                            rendered_keys: keycaps.length,
                            theme: bodyTheme,
                            active_cmd: preview.textContent
                        }
                    });
                } catch (e) {
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'phase3_test',
                        action: 'phase3_result',
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
                # Only handle phase3_result!
                if data.get("action") == "phase3_result":
                    payload = data.get("payload", {})
                    self.test_passed = payload.get("success", False)
                    self.error_message = payload.get("error")
                    self.diagnostics = payload
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

        print(f"Phase 3 Diagnostics: {self.diagnostics}")
        self.assertTrue(self.test_passed, f"Phase 3 UI test failed: {self.error_message}")

if __name__ == "__main__":
    unittest.main()
