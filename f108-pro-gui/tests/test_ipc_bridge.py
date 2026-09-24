#!/usr/bin/env python3
"""
Test Suite for AULA F108 Pro Native Window & IPC Bridge (Phase 2).
Automates launching the WebKit window, issuing IPC requests via JavaScript,
and validating that responses return successfully.
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

class TestIPCBridge(unittest.TestCase):
    def setUp(self):
        self.app = KeyboardManagerApp()
        self.test_passed = False
        self.error_message = None

    def test_ipc_communication(self):
        # We inject a test script into WebView that tests all IPC calls
        def run_test():
            test_script = """
            (async function() {
                try {
                    // 1. Test get_app_info
                    const appInfo = await window.hostBridge.invoke('get_app_info');
                    if (!appInfo || !appInfo.cli_path) throw new Error('Invalid app info');

                    // 2. Test get_device_status
                    const status = await window.hostBridge.invoke('get_device_status');
                    if (typeof status.connected !== 'boolean') throw new Error('Invalid device status');

                    // 3. Test execute_cli (--version)
                    const cliRes = await window.hostBridge.invoke('execute_cli', { args: ['--version'] });
                    if (cliRes.exit_code !== 0) throw new Error('CLI failed: ' + cliRes.stderr);

                    // Notify Python that test passed
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'test_done',
                        action: 'test_result',
                        payload: { success: true }
                    });
                } catch (e) {
                    window.webkit.messageHandlers.api.postMessage({
                        id: 'test_done',
                        action: 'test_result',
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
                if data.get("action") == "test_result":
                    payload = data.get("payload", {})
                    self.test_passed = payload.get("success", False)
                    self.error_message = payload.get("error")
                    Gtk.main_quit()
            except Exception as e:
                self.error_message = str(e)
                Gtk.main_quit()

        self.app.ucm.connect("script-message-received::api", on_msg)

        # Trigger test after load finishes
        def on_load_changed(webview, event):
            if event == WebKit2.LoadEvent.FINISHED:
                GLib.timeout_add(300, run_test)

        self.app.webview.connect("load-changed", on_load_changed)

        # Safety timeout after 8 seconds
        GLib.timeout_add_seconds(8, Gtk.main_quit)

        self.app.show_all()
        Gtk.main()

        self.assertTrue(self.test_passed, f"IPC test failed: {self.error_message}")

if __name__ == "__main__":
    unittest.main()
