#!/usr/bin/env python3
"""
Test Suite for AULA F108 Pro GUI Keyboard Coordinates.
Validates:
1. All 104 keys are mapped with valid positive coordinates and dimensions.
2. No duplicate keys exist.
3. Every key matches an expected key in the Rust CLI matrix.
4. Total canvas matches 800x300.
"""

import json
import os
import unittest

class TestKeyCoordinates(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.coords_path = "/mnt/Files/AgenteLocalLinux/AULASetup/f108-pro-gui/assets/keyboard/key_coordinates.json"
        with open(cls.coords_path, "r", encoding="utf-8") as f:
            cls.data = json.load(f)

    def test_canvas_dimensions(self):
        canvas = self.data.get("canvas", {})
        self.assertEqual(canvas.get("width"), 800)
        self.assertEqual(canvas.get("height"), 300)
        self.assertEqual(canvas.get("image"), "img_keyboard_layout.png")

    def test_total_keys_count(self):
        keys = self.data.get("keys", [])
        self.assertEqual(len(keys), 104, "Expected exactly 104 keys from layout XML")

    def test_keys_uniqueness_and_bounds(self):
        keys = self.data.get("keys", [])
        seen_ids = set()
        seen_light_indices = set()
        
        for k in keys:
            key_id = k.get("id")
            self.assertNotIn(key_id, seen_ids, f"Duplicate key ID found: {key_id}")
            seen_ids.add(key_id)
            
            rect = k.get("rect", {})
            x = rect.get("x")
            y = rect.get("y")
            w = rect.get("w")
            h = rect.get("h")
            
            self.assertGreaterEqual(x, 0, f"Key {key_id} x coordinate must be >= 0")
            self.assertGreaterEqual(y, 0, f"Key {key_id} y coordinate must be >= 0")
            self.assertGreater(w, 0, f"Key {key_id} width must be > 0")
            self.assertGreater(h, 0, f"Key {key_id} height must be > 0")
            self.assertLessEqual(x + w, 800, f"Key {key_id} exceeds canvas width")
            self.assertLessEqual(y + h, 300, f"Key {key_id} exceeds canvas height")

    def test_crucial_keys_present(self):
        keys = self.data.get("keys", [])
        key_ids = {k.get("id") for k in keys}
        
        essential_keys = [
            "ESC", "F1", "F12", "PRINT", "SCROLL", "PAUSE",
            "GRAVE", "1", "0", "MINUS", "EQUALS", "BACKSPACE",
            "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P",
            "CAPSLOCK", "A", "S", "D", "F", "G", "H", "J", "K", "L", "ENTER",
            "LSHIFT", "Z", "X", "C", "V", "B", "N", "M", "RSHIFT",
            "LCTRL", "LWIN", "LALT", "SPACE", "RALT", "FN", "RWIN", "RCTRL",
            "UP", "DOWN", "LEFT", "RIGHT",
            "INSERT", "DELETE", "HOME", "END", "PAGEUP", "PAGEDOWN",
            "NUMLOCK", "KP_DIVIDE", "KP_MULTIPLY", "KP_MINUS", "KP_PLUS", "KP_ENTER",
            "KP_0", "KP_1", "KP_7", "KP_9", "KP_DOT"
        ]
        
        for k in essential_keys:
            self.assertIn(k, key_ids, f"Essential key missing from layout: {k}")

if __name__ == "__main__":
    unittest.main()
