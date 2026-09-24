#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AULA F108 Pro Keyboard Manager - Native Linux Application Host
==============================================================
Provides GTK3 window, WebKitGTK 4.1 engine, IPC bridge,
and integration with f108-pro and mkimage.
"""

import os
import sys
import json
import glob
import shutil
import tempfile
import threading
import subprocess
from pathlib import Path
from PIL import Image

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
gi.require_version('WebKit2', '4.1')
gi.require_version('JavaScriptCore', '4.1')
from gi.repository import Gtk, Gdk, WebKit2, GLib

if getattr(sys, 'frozen', False):
    APP_DIR = Path(sys._MEIPASS)
else:
    APP_DIR = Path(__file__).resolve().parent
UI_DIR = APP_DIR / "ui"
ASSETS_DIR = APP_DIR / "assets"

def find_binary(name):
    candidates = [
        shutil.which(name),
        os.path.expanduser(f"~/.local/bin/{name}"),
        f"/usr/local/bin/{name}",
        f"/mnt/Files/AgenteLocalLinux/AULASetup/f108-pro-rust/target/release/{name}"
    ]
    for c in candidates:
        if c and os.path.isfile(c) and os.access(c, os.X_OK):
            return str(Path(c).resolve())
    return name

class KeyboardManagerApp(Gtk.Window):
    def __init__(self):
        super().__init__(title="AULA F108 Pro Keyboard Manager")
        self.cli_binary = find_binary("f108-pro")
        self.mkimage_binary = find_binary("mkimage")
        
        self.set_default_size(1140, 800)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.connect("destroy", Gtk.main_quit)

        icon_path = ASSETS_DIR / "icons" / "tab_customkey.png"
        if icon_path.exists():
            self.set_icon_from_file(str(icon_path))

        self.ucm = WebKit2.UserContentManager()
        self.ucm.register_script_message_handler("api")
        self.ucm.connect("script-message-received::api", self.on_script_message)

        settings = WebKit2.Settings()
        settings.set_enable_developer_extras(True)
        settings.set_enable_webgl(True)
        settings.set_enable_smooth_scrolling(True)
        settings.set_allow_file_access_from_file_urls(True)
        settings.set_allow_universal_access_from_file_urls(True)

        self.webview = WebKit2.WebView.new_with_user_content_manager(self.ucm)
        self.webview.set_settings(settings)

        scrolled = Gtk.ScrolledWindow()
        scrolled.add(self.webview)
        self.add(scrolled)

        index_uri = (UI_DIR / "index.html").as_uri()
        self.webview.load_uri(index_uri)

        GLib.timeout_add_seconds(3, self.check_usb_status_periodic)

    def on_script_message(self, user_content_manager, js_result):
        try:
            js_val = js_result.get_js_value()
            json_str = js_val.to_json(0)
            data = json.loads(json_str)
            req_id = data.get("id")
            action = data.get("action")
            payload = data.get("payload", {})
            self.route_request(req_id, action, payload)
        except Exception as e:
            print(f"[IPC Error] {e}", file=sys.stderr)

    def route_request(self, req_id, action, payload):
        if action == "execute_cli":
            threading.Thread(target=self.handle_execute_cli, args=(req_id, payload), daemon=True).start()
        elif action == "get_device_status":
            self.send_response(req_id, True, self.query_device_status())
        elif action == "get_app_info":
            info = {
                "version": "1.0.0",
                "cli_path": self.cli_binary,
                "mkimage_path": self.mkimage_binary,
                "cli_exists": os.path.isfile(self.cli_binary),
                "python_version": sys.version.split()[0],
                "ui_dir": str(UI_DIR)
            }
            self.send_response(req_id, True, info)
        elif action == "convert_and_flash_lcd":
            threading.Thread(target=self.handle_convert_and_flash_lcd, args=(req_id, payload), daemon=True).start()
        elif action == "open_file_dialog":
            self.handle_open_file_dialog(req_id, payload)
        elif action == "save_file_dialog":
            self.handle_save_file_dialog(req_id, payload)
        elif action == "read_file":
            self.handle_read_file(req_id, payload)
        elif action == "write_file":
            self.handle_write_file(req_id, payload)
        else:
            self.send_response(req_id, False, error=f"Unknown action: {action}")

    def handle_execute_cli(self, req_id, payload):
        cmd_args = payload.get("args", [])
        raw_cmd = payload.get("command", "")

        if raw_cmd and not cmd_args:
            parts = raw_cmd.strip().split()
            if parts and parts[0] in ["f108-pro", self.cli_binary, os.path.basename(self.cli_binary)]:
                cmd = [self.cli_binary] + parts[1:]
            else:
                cmd = [self.cli_binary] + parts
        else:
            cmd = [self.cli_binary] + cmd_args

        self.emit_event("cli_log", {"stream": "info", "text": f"$ {' '.join(cmd)}\n"})

        try:
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1
            )

            stdout_lines = []
            stderr_lines = []

            def read_stream(stream, stream_name, accumulator):
                for line in iter(stream.readline, ""):
                    accumulator.append(line)
                    self.emit_event("cli_log", {"stream": stream_name, "text": line})
                stream.close()

            t1 = threading.Thread(target=read_stream, args=(process.stdout, "stdout", stdout_lines))
            t2 = threading.Thread(target=read_stream, args=(process.stderr, "stderr", stderr_lines))
            t1.start()
            t2.start()
            t1.join()
            t2.join()

            exit_code = process.wait()
            success = (exit_code == 0)
            result = {
                "exit_code": exit_code,
                "stdout": "".join(stdout_lines),
                "stderr": "".join(stderr_lines)
            }
            self.send_response(req_id, success, result, error=None if success else f"Exited with code {exit_code}")
        except Exception as e:
            self.emit_event("cli_log", {"stream": "stderr", "text": f"[ERROR] {e}\n"})
            self.send_response(req_id, False, error=str(e))

    def handle_convert_and_flash_lcd(self, req_id, payload):
        image_path = payload.get("image_path")
        if not image_path or not os.path.isfile(image_path):
            self.send_response(req_id, False, error="Invalid image path")
            return

        try:
            self.emit_event("cli_log", {"stream": "info", "text": f"[LCD] Processing image: {image_path}\n"})
            tmp_dir = tempfile.mkdtemp(prefix="f108_lcd_")
            tmp_gif = os.path.join(tmp_dir, "frames_240x135.gif")
            tmp_bin = os.path.join(tmp_dir, "lcd_display.bin")

            # Load and resize with Pillow
            with Image.open(image_path) as im:
                frames = []
                frame_count = 0
                try:
                    while True:
                        frame = im.copy().convert("RGB")
                        frame = frame.resize((240, 135), Image.Resampling.LANCZOS)
                        frames.append(frame)
                        frame_count += 1
                        im.seek(im.tell() + 1)
                except EOFError:
                    pass

                if frames:
                    frames[0].save(
                        tmp_gif,
                        save_all=True,
                        append_images=frames[1:],
                        loop=0,
                        duration=im.info.get('duration', 100)
                    )

            self.emit_event("cli_log", {"stream": "info", "text": f"[LCD] Generated {len(frames)} resized frames. Compiling with mkimage...\n"})
            
            # Run mkimage
            mk_cmd = [self.mkimage_binary, "--gif", tmp_gif, "-o", tmp_bin]
            res_mk = subprocess.run(mk_cmd, capture_output=True, text=True)
            if res_mk.returncode != 0:
                raise Exception(f"mkimage failed: {res_mk.stderr}")

            # Run f108-pro lcd
            self.emit_event("cli_log", {"stream": "info", "text": f"$ {self.cli_binary} lcd {tmp_bin}\n"})
            lcd_cmd = [self.cli_binary, "lcd", tmp_bin]
            res_lcd = subprocess.run(lcd_cmd, capture_output=True, text=True)
            if res_lcd.returncode != 0:
                raise Exception(f"f108-pro lcd failed: {res_lcd.stderr}")

            self.emit_event("cli_log", {"stream": "stdout", "text": res_lcd.stdout})
            self.send_response(req_id, True, {"success": True, "frames": len(frames)})
        except Exception as e:
            self.emit_event("cli_log", {"stream": "stderr", "text": f"[LCD ERROR] {e}\n"})
            self.send_response(req_id, False, error=str(e))

    def query_device_status(self):
        status = {
            "connected": False,
            "mode": "none",
            "cable_connected": False,
            "dongle_connected": False,
            "details": "No keyboard detected"
        }
        try:
            for uevent in glob.glob("/sys/bus/usb/devices/*/uevent"):
                try:
                    with open(uevent, "r", encoding="utf-8") as f:
                        content = f.read().upper()
                        if "0C45/800A" in content or "PRODUCT=C45/800A" in content:
                            status["cable_connected"] = True
                        if "05AC/024F" in content or "PRODUCT=5AC/24F" in content:
                            status["dongle_connected"] = True
                except Exception:
                    pass

            if status["cable_connected"]:
                status["connected"] = True
                status["mode"] = "wired"
                status["details"] = "Connected via USB Cable (Full feature reports ready)"
            elif status["dongle_connected"]:
                status["connected"] = True
                status["mode"] = "wireless"
                status["details"] = "Connected via 2.4G Dongle (Connect USB cable for configuration)"
        except Exception as e:
            status["details"] = f"Detection error: {e}"

        return status

    def check_usb_status_periodic(self):
        self.emit_event("usb_status_changed", self.query_device_status())
        return True

    def handle_open_file_dialog(self, req_id, payload):
        title = payload.get("title", "Select File")
        filter_type = payload.get("filter", "all")

        dialog = Gtk.FileChooserDialog(title=title, parent=self, action=Gtk.FileChooserAction.OPEN)
        dialog.add_buttons(Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL, Gtk.STOCK_OPEN, Gtk.ResponseType.ACCEPT)

        if filter_type == "image":
            f = Gtk.FileFilter()
            f.set_name("Images & GIFs (*.png, *.jpg, *.gif)")
            f.add_mime_type("image/png")
            f.add_mime_type("image/jpeg")
            f.add_mime_type("image/gif")
            dialog.add_filter(f)
        elif filter_type == "yaml":
            f = Gtk.FileFilter()
            f.set_name("YAML Profiles (*.yaml, *.yml)")
            f.add_pattern("*.yaml")
            f.add_pattern("*.yml")
            dialog.add_filter(f)

        f_all = Gtk.FileFilter()
        f_all.set_name("All Files (*.*)")
        f_all.add_pattern("*")
        dialog.add_filter(f_all)

        response = dialog.run()
        selected_file = dialog.get_filename() if response == Gtk.ResponseType.ACCEPT else None
        dialog.destroy()

        if selected_file:
            self.send_response(req_id, True, {"path": selected_file})
        else:
            self.send_response(req_id, False, error="Dialog cancelled")

    def handle_save_file_dialog(self, req_id, payload):
        title = payload.get("title", "Save File")
        default_name = payload.get("default_name", "profile.yaml")

        dialog = Gtk.FileChooserDialog(title=title, parent=self, action=Gtk.FileChooserAction.SAVE)
        dialog.add_buttons(Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL, Gtk.STOCK_SAVE, Gtk.ResponseType.ACCEPT)
        dialog.set_current_name(default_name)
        dialog.set_do_overwrite_confirmation(True)

        response = dialog.run()
        selected_file = dialog.get_filename() if response == Gtk.ResponseType.ACCEPT else None
        dialog.destroy()

        if selected_file:
            self.send_response(req_id, True, {"path": selected_file})
        else:
            self.send_response(req_id, False, error="Dialog cancelled")

    def handle_read_file(self, req_id, payload):
        path = payload.get("path")
        try:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            self.send_response(req_id, True, {"content": content})
        except Exception as e:
            self.send_response(req_id, False, error=str(e))

    def handle_write_file(self, req_id, payload):
        path = payload.get("path")
        content = payload.get("content", "")
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            self.send_response(req_id, True, {"path": path})
        except Exception as e:
            self.send_response(req_id, False, error=str(e))

    def send_response(self, req_id, success, data=None, error=None):
        payload = json.dumps({
            "id": req_id,
            "success": success,
            "data": data,
            "error": error
        })
        script = f"window.__onApiResponse && window.__onApiResponse({payload});"
        GLib.idle_add(lambda: self.webview.evaluate_javascript(script, -1, None, None, None, None, None))

    def emit_event(self, event_name, data):
        payload = json.dumps(data)
        script = f"window.__onApiEvent && window.__onApiEvent('{event_name}', {payload});"
        GLib.idle_add(lambda: self.webview.evaluate_javascript(script, -1, None, None, None, None, None))

def main():
    Gtk.init(None)
    app = KeyboardManagerApp()
    app.show_all()
    Gtk.main()

if __name__ == "__main__":
    main()
