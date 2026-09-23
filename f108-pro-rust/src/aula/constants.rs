//! # AULA F108 Pro - Driver Constants & Protocol Definitions
//!
//! This module defines the low-level constants, USB endpoints, vendor/product IDs,
//! command opcodes, physical/logical key mappings, consumer controls, mouse actions,
//! modifier keys, and lighting mode enumerations for the AULA F108 Pro mechanical keyboard.
//!
//! ## Hardware Architecture & Protocol Overview
//! - **MCU / Controller**: Sonix / Microdia USB Keyboard & Display MCU.
//! - **USB Interfaces**:
//!   - Interface 3: Control & Feature Reports (64 bytes payload over HID Set/Get Report).
//!   - Interface 2: TFT LCD display streaming (Interrupt OUT endpoint 3, Interrupt IN endpoint 4 for ACKs, 4096-byte pages).
//! - **Timing constraints**: Firmware requires a mandatory delay of **35ms** between consecutive command transactions to prevent buffer overflows or lockups.
//! - **Transaction Flow**: `OPCODE_BEGIN` (0x04 0x18) -> Payload/Init -> `OPCODE_APPLY` (0x04 0x02) -> `OPCODE_FINALIZE` (0x04 0xF0) terminated with `TRAILER_55AA` (0x55 0xAA).

// ============================================================================
// USB IDENTIFIERS
// ============================================================================

pub const VENDOR_ID_WIRED: u16 = 0x0C45;  // Sonix Technology Co., Ltd. (Vivitar Vivicam kernel detection)
pub const PRODUCT_ID_WIRED: u16 = 0x800A; // AULA F108 Pro Wired Mechanical Keyboard
pub const VENDOR_ID_WIRELESS: u16 = 0x05AC; // Wireless 2.4GHz Dongle Vendor ID
pub const PRODUCT_ID_WIRELESS: u16 = 0x024F; // Wireless 2.4GHz Dongle Product ID

// ============================================================================
// USB HID & ENDPOINT CONSTANTS
// ============================================================================

pub const INTERFACE_CONTROL: u8 = 3;
pub const INTERFACE_LCD: u8 = 2;
pub const LCD_OUT_EP: u8 = 3;
pub const LCD_IN_EP: u8 = 4;
pub const LCD_PAGE_SIZE: usize = 4096;
pub const REPORT_SIZE: usize = 64;
pub const CMD_DELAY_MS: u64 = 35;

pub const REQ_TYPE_OUT: u8 = 0x21; // Host-to-device, Class, Interface
pub const REQ_TYPE_IN: u8 = 0xA1;  // Device-to-host, Class, Interface
pub const REQ_SET_REPORT: u8 = 0x09;
pub const REQ_GET_REPORT: u8 = 0x01;
pub const FEATURE_REPORT_TYPE: u16 = 0x0300; // Feature report type 3, report ID 0

// ============================================================================
// PROTOCOL OPCODES & MARKERS
// ============================================================================

pub const OPCODE_BEGIN: [u8; 2] = [0x04, 0x18];
pub const OPCODE_APPLY: [u8; 2] = [0x04, 0x02];
pub const OPCODE_FINALIZE: [u8; 2] = [0x04, 0xF0];
pub const OPCODE_LIGHT_INIT: [u8; 2] = [0x04, 0x13];
pub const OPCODE_CLOCK_INIT: [u8; 2] = [0x04, 0x28];
pub const OPCODE_LCD_HEADER: [u8; 2] = [0x04, 0x72];
pub const TRAILER_55AA: [u8; 2] = [0x55, 0xAA];
pub const MAGIC_CLOCK_MARKER: u8 = 0x5A;

// ============================================================================
// LIGHTING MODES (0 - 19)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    Off = 0,
    Static = 1,
    SingleOn = 2,
    SingleOff = 3,
    Glittering = 4,
    Falling = 5,
    Colourful = 6,
    Breath = 7,
    Spectrum = 8,
    Outward = 9,
    Scrolling = 10,
    Rolling = 11,
    Rotating = 12,
    Explode = 13,
    Launch = 14,
    Ripples = 15,
    Flowing = 16,
    Pulsating = 17,
    Tilt = 18,
    Shuttle = 19,
}

impl LightingMode {
    pub fn name(&self) -> &'static str {
        match self {
            LightingMode::Off => "Off",
            LightingMode::Static => "Static",
            LightingMode::SingleOn => "SingleOn",
            LightingMode::SingleOff => "SingleOff",
            LightingMode::Glittering => "Glittering",
            LightingMode::Falling => "Falling",
            LightingMode::Colourful => "Colourful",
            LightingMode::Breath => "Breath",
            LightingMode::Spectrum => "Spectrum",
            LightingMode::Outward => "Outward",
            LightingMode::Scrolling => "Scrolling",
            LightingMode::Rolling => "Rolling",
            LightingMode::Rotating => "Rotating",
            LightingMode::Explode => "Explode",
            LightingMode::Launch => "Launch",
            LightingMode::Ripples => "Ripples",
            LightingMode::Flowing => "Flowing",
            LightingMode::Pulsating => "Pulsating",
            LightingMode::Tilt => "Tilt",
            LightingMode::Shuttle => "Shuttle",
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::Static),
            2 => Some(Self::SingleOn),
            3 => Some(Self::SingleOff),
            4 => Some(Self::Glittering),
            5 => Some(Self::Falling),
            6 => Some(Self::Colourful),
            7 => Some(Self::Breath),
            8 => Some(Self::Spectrum),
            9 => Some(Self::Outward),
            10 => Some(Self::Scrolling),
            11 => Some(Self::Rolling),
            12 => Some(Self::Rotating),
            13 => Some(Self::Explode),
            14 => Some(Self::Launch),
            15 => Some(Self::Ripples),
            16 => Some(Self::Flowing),
            17 => Some(Self::Pulsating),
            18 => Some(Self::Tilt),
            19 => Some(Self::Shuttle),
            _ => None,
        }
    }
}

// ============================================================================
// PHYSICAL KEY INDICES & MAPPINGS (1 to 132 for 108/F108 layout)
// ============================================================================

pub const KEY_ESC: u8 = 1;
pub const KEY_F1: u8 = 2;
pub const KEY_F2: u8 = 3;
pub const KEY_F3: u8 = 4;
pub const KEY_F4: u8 = 5;
pub const KEY_F5: u8 = 6;
pub const KEY_F6: u8 = 7;
pub const KEY_F7: u8 = 8;
pub const KEY_F8: u8 = 9;
pub const KEY_F9: u8 = 10;
pub const KEY_F10: u8 = 11;
pub const KEY_F11: u8 = 12;
pub const KEY_F12: u8 = 13;
pub const KEY_PRINT: u8 = 14;
pub const KEY_SCROLL: u8 = 15;
pub const KEY_PAUSE: u8 = 16;

pub const KEY_GRAVE: u8 = 17;
pub const KEY_1: u8 = 18;
pub const KEY_2: u8 = 19;
pub const KEY_3: u8 = 20;
pub const KEY_4: u8 = 21;
pub const KEY_5: u8 = 22;
pub const KEY_6: u8 = 23;
pub const KEY_7: u8 = 24;
pub const KEY_8: u8 = 25;
pub const KEY_9: u8 = 26;
pub const KEY_0: u8 = 27;
pub const KEY_MINUS: u8 = 28;
pub const KEY_EQUALS: u8 = 29;
pub const KEY_BACKSPACE: u8 = 30;
pub const KEY_INSERT: u8 = 31;
pub const KEY_HOME: u8 = 32;
pub const KEY_PAGEUP: u8 = 33;
pub const KEY_NUMLOCK: u8 = 34;
pub const KEY_KP_DIVIDE: u8 = 35;
pub const KEY_KP_MULTIPLY: u8 = 36;
pub const KEY_KP_MINUS: u8 = 37;

pub const KEY_TAB: u8 = 38;
pub const KEY_Q: u8 = 39;
pub const KEY_W: u8 = 40;
pub const KEY_E: u8 = 41;
pub const KEY_R: u8 = 42;
pub const KEY_T: u8 = 43;
pub const KEY_Y: u8 = 44;
pub const KEY_U: u8 = 45;
pub const KEY_I: u8 = 46;
pub const KEY_O: u8 = 47;
pub const KEY_P: u8 = 48;
pub const KEY_LBRACKET: u8 = 49;
pub const KEY_RBRACKET: u8 = 50;
pub const KEY_BACKSLASH: u8 = 51;
pub const KEY_DELETE: u8 = 52;
pub const KEY_END: u8 = 53;
pub const KEY_PAGEDOWN: u8 = 54;
pub const KEY_KP_7: u8 = 55;
pub const KEY_KP_8: u8 = 56;
pub const KEY_KP_9: u8 = 57;
pub const KEY_KP_PLUS: u8 = 58;

pub const KEY_CAPSLOCK: u8 = 59;
pub const KEY_A: u8 = 60;
pub const KEY_S: u8 = 61;
pub const KEY_D: u8 = 62;
pub const KEY_F: u8 = 63;
pub const KEY_G: u8 = 64;
pub const KEY_H: u8 = 65;
pub const KEY_J: u8 = 66;
pub const KEY_K: u8 = 67;
pub const KEY_L: u8 = 68;
pub const KEY_SEMICOLON: u8 = 69;
pub const KEY_QUOTE: u8 = 70;
pub const KEY_ENTER: u8 = 71;
pub const KEY_KP_4: u8 = 72;
pub const KEY_KP_5: u8 = 73;
pub const KEY_KP_6: u8 = 74;

pub const KEY_LSHIFT: u8 = 75;
pub const KEY_Z: u8 = 76;
pub const KEY_X: u8 = 77;
pub const KEY_C: u8 = 78;
pub const KEY_V: u8 = 79;
pub const KEY_B: u8 = 80;
pub const KEY_N: u8 = 81;
pub const KEY_M: u8 = 82;
pub const KEY_COMMA: u8 = 83;
pub const KEY_DOT: u8 = 84;
pub const KEY_SLASH: u8 = 85;
pub const KEY_RSHIFT: u8 = 86;
pub const KEY_UP: u8 = 87;
pub const KEY_KP_1: u8 = 88;
pub const KEY_KP_2: u8 = 89;
pub const KEY_KP_3: u8 = 90;
pub const KEY_KP_ENTER: u8 = 91;

pub const KEY_LCTRL: u8 = 92;
pub const KEY_LWIN: u8 = 93;
pub const KEY_LALT: u8 = 94;
pub const KEY_SPACE: u8 = 95;
pub const KEY_RALT: u8 = 96;
pub const KEY_RWIN: u8 = 97;
pub const KEY_FN: u8 = 98;
pub const KEY_RCTRL: u8 = 99;
pub const KEY_LEFT: u8 = 100;
pub const KEY_DOWN: u8 = 101;
pub const KEY_RIGHT: u8 = 102;
pub const KEY_KP_0: u8 = 103;
pub const KEY_KP_DOT: u8 = 104;

pub const KEY_MUTE: u8 = 105;
pub const KEY_VOL_DOWN: u8 = 106;
pub const KEY_VOL_UP: u8 = 107;
pub const KEY_CALCULATOR: u8 = 108;

/// Maps a human-readable key name to its physical index in the LED/matrix grid (1..=132).
pub fn key_name_to_index(name: &str) -> Option<u8> {
    match name.to_uppercase().as_str() {
        "ESC" => Some(KEY_ESC),
        "F1" => Some(KEY_F1),
        "F2" => Some(KEY_F2),
        "F3" => Some(KEY_F3),
        "F4" => Some(KEY_F4),
        "F5" => Some(KEY_F5),
        "F6" => Some(KEY_F6),
        "F7" => Some(KEY_F7),
        "F8" => Some(KEY_F8),
        "F9" => Some(KEY_F9),
        "F10" => Some(KEY_F10),
        "F11" => Some(KEY_F11),
        "F12" => Some(KEY_F12),
        "PRINTSCREEN" | "PRINT" => Some(KEY_PRINT),
        "SCROLLLOCK" | "SCROLL" => Some(KEY_SCROLL),
        "PAUSE" => Some(KEY_PAUSE),
        "GRAVE" | "`" => Some(KEY_GRAVE),
        "1" => Some(KEY_1),
        "2" => Some(KEY_2),
        "3" => Some(KEY_3),
        "4" => Some(KEY_4),
        "5" => Some(KEY_5),
        "6" => Some(KEY_6),
        "7" => Some(KEY_7),
        "8" => Some(KEY_8),
        "9" => Some(KEY_9),
        "0" => Some(KEY_0),
        "MINUS" | "-" => Some(KEY_MINUS),
        "EQUALS" | "=" => Some(KEY_EQUALS),
        "BACKSPACE" => Some(KEY_BACKSPACE),
        "INSERT" => Some(KEY_INSERT),
        "HOME" => Some(KEY_HOME),
        "PAGEUP" => Some(KEY_PAGEUP),
        "NUMLOCK" => Some(KEY_NUMLOCK),
        "KP_DIVIDE" | "KP/" => Some(KEY_KP_DIVIDE),
        "KP_MULTIPLY" | "KP*" => Some(KEY_KP_MULTIPLY),
        "KP_MINUS" | "KP-" => Some(KEY_KP_MINUS),
        "TAB" => Some(KEY_TAB),
        "Q" => Some(KEY_Q),
        "W" => Some(KEY_W),
        "E" => Some(KEY_E),
        "R" => Some(KEY_R),
        "T" => Some(KEY_T),
        "Y" => Some(KEY_Y),
        "U" => Some(KEY_U),
        "I" => Some(KEY_I),
        "O" => Some(KEY_O),
        "P" => Some(KEY_P),
        "LBRACKET" | "[" => Some(KEY_LBRACKET),
        "RBRACKET" | "]" => Some(KEY_RBRACKET),
        "BACKSLASH" | "\\" => Some(KEY_BACKSLASH),
        "DELETE" => Some(KEY_DELETE),
        "END" => Some(KEY_END),
        "PAGEDOWN" => Some(KEY_PAGEDOWN),
        "KP_7" => Some(KEY_KP_7),
        "KP_8" => Some(KEY_KP_8),
        "KP_9" => Some(KEY_KP_9),
        "KP_PLUS" | "KP+" => Some(KEY_KP_PLUS),
        "CAPSLOCK" => Some(KEY_CAPSLOCK),
        "A" => Some(KEY_A),
        "S" => Some(KEY_S),
        "D" => Some(KEY_D),
        "F" => Some(KEY_F),
        "G" => Some(KEY_G),
        "H" => Some(KEY_H),
        "J" => Some(KEY_J),
        "K" => Some(KEY_K),
        "L" => Some(KEY_L),
        "SEMICOLON" | ";" => Some(KEY_SEMICOLON),
        "QUOTE" | "'" => Some(KEY_QUOTE),
        "ENTER" => Some(KEY_ENTER),
        "KP_4" => Some(KEY_KP_4),
        "KP_5" => Some(KEY_KP_5),
        "KP_6" => Some(KEY_KP_6),
        "LSHIFT" => Some(KEY_LSHIFT),
        "Z" => Some(KEY_Z),
        "X" => Some(KEY_X),
        "C" => Some(KEY_C),
        "V" => Some(KEY_V),
        "B" => Some(KEY_B),
        "N" => Some(KEY_N),
        "M" => Some(KEY_M),
        "COMMA" | "," => Some(KEY_COMMA),
        "DOT" | "." => Some(KEY_DOT),
        "SLASH" | "/" => Some(KEY_SLASH),
        "RSHIFT" => Some(KEY_RSHIFT),
        "UP" => Some(KEY_UP),
        "KP_1" => Some(KEY_KP_1),
        "KP_2" => Some(KEY_KP_2),
        "KP_3" => Some(KEY_KP_3),
        "KP_ENTER" => Some(KEY_KP_ENTER),
        "LCTRL" => Some(KEY_LCTRL),
        "LWIN" | "LGUI" => Some(KEY_LWIN),
        "LALT" => Some(KEY_LALT),
        "SPACE" => Some(KEY_SPACE),
        "RALT" => Some(KEY_RALT),
        "RWIN" | "RGUI" => Some(KEY_RWIN),
        "FN" => Some(KEY_FN),
        "RCTRL" => Some(KEY_RCTRL),
        "LEFT" => Some(KEY_LEFT),
        "DOWN" => Some(KEY_DOWN),
        "RIGHT" => Some(KEY_RIGHT),
        "KP_0" => Some(KEY_KP_0),
        "KP_DOT" => Some(KEY_KP_DOT),
        _ => None,
    }
}

/// Maps a key name to its USB HID Usage ID (Standard Keyboard/Keypad page 0x07).
pub fn key_name_to_hid(name: &str) -> Option<u8> {
    match name.to_uppercase().as_str() {
        "A" => Some(0x04),
        "B" => Some(0x05),
        "C" => Some(0x06),
        "D" => Some(0x07),
        "E" => Some(0x08),
        "F" => Some(0x09),
        "G" => Some(0x0A),
        "H" => Some(0x0B),
        "I" => Some(0x0C),
        "J" => Some(0x0D),
        "K" => Some(0x0E),
        "L" => Some(0x0F),
        "M" => Some(0x10),
        "N" => Some(0x11),
        "O" => Some(0x12),
        "P" => Some(0x13),
        "Q" => Some(0x14),
        "R" => Some(0x15),
        "S" => Some(0x16),
        "T" => Some(0x17),
        "U" => Some(0x18),
        "V" => Some(0x19),
        "W" => Some(0x1A),
        "X" => Some(0x1B),
        "Y" => Some(0x1C),
        "Z" => Some(0x1D),
        "1" => Some(0x1E),
        "2" => Some(0x1F),
        "3" => Some(0x20),
        "4" => Some(0x21),
        "5" => Some(0x22),
        "6" => Some(0x23),
        "7" => Some(0x24),
        "8" => Some(0x25),
        "9" => Some(0x26),
        "0" => Some(0x27),
        "ENTER" => Some(0x28),
        "ESC" => Some(0x29),
        "BACKSPACE" => Some(0x2A),
        "TAB" => Some(0x2B),
        "SPACE" => Some(0x2C),
        "MINUS" | "-" => Some(0x2D),
        "EQUALS" | "=" => Some(0x2E),
        "LBRACKET" | "[" => Some(0x2F),
        "RBRACKET" | "]" => Some(0x30),
        "BACKSLASH" | "\\" => Some(0x31),
        "SEMICOLON" | ";" => Some(0x33),
        "QUOTE" | "'" => Some(0x34),
        "GRAVE" | "`" => Some(0x35),
        "COMMA" | "," => Some(0x36),
        "DOT" | "." => Some(0x37),
        "SLASH" | "/" => Some(0x38),
        "CAPSLOCK" => Some(0x39),
        "F1" => Some(0x3A),
        "F2" => Some(0x3B),
        "F3" => Some(0x3C),
        "F4" => Some(0x3D),
        "F5" => Some(0x3E),
        "F6" => Some(0x3F),
        "F7" => Some(0x40),
        "F8" => Some(0x41),
        "F9" => Some(0x42),
        "F10" => Some(0x43),
        "F11" => Some(0x44),
        "F12" => Some(0x45),
        "PRINT" => Some(0x46),
        "SCROLL" => Some(0x47),
        "PAUSE" => Some(0x48),
        "INSERT" => Some(0x49),
        "HOME" => Some(0x4A),
        "PAGEUP" => Some(0x4B),
        "DELETE" => Some(0x4C),
        "END" => Some(0x4D),
        "PAGEDOWN" => Some(0x4E),
        "RIGHT" => Some(0x4F),
        "LEFT" => Some(0x50),
        "DOWN" => Some(0x51),
        "UP" => Some(0x52),
        _ => None,
    }
}

/// Maps consumer control action names to protocol consumer codes.
pub fn consumer_name_to_code(name: &str) -> Option<u8> {
    match name.to_uppercase().as_str() {
        "MUTE" => Some(0xE2),
        "VOL_UP" | "VOLUMEUP" => Some(0xE9),
        "VOL_DOWN" | "VOLUMEDOWN" => Some(0xEA),
        "PLAY_PAUSE" => Some(0xCD),
        "STOP" => Some(0xB7),
        "NEXT_TRACK" => Some(0xB5),
        "PREV_TRACK" => Some(0xB6),
        "CALCULATOR" => Some(0x92),
        "MY_COMPUTER" => Some(0x94),
        "EMAIL" => Some(0x8A),
        "WWW_BROWSER" => Some(0x23),
        _ => None,
    }
}

/// Maps mouse emulation action names to command parameters [button, action, wheel_delta].
pub fn mouse_name_to_params(name: &str) -> Option<[u8; 3]> {
    match name.to_uppercase().as_str() {
        "MOUSE_LEFT" => Some([0x01, 0x01, 0x00]),
        "MOUSE_RIGHT" => Some([0x02, 0x01, 0x00]),
        "MOUSE_MIDDLE" => Some([0x04, 0x01, 0x00]),
        "WHEEL_UP" => Some([0x00, 0x00, 0x01]),
        "WHEEL_DOWN" => Some([0x00, 0x00, 0xFF]), // -1
        _ => None,
    }
}

/// Checks if a given HID usage code corresponds to a modifier key.
pub fn is_modifier_key(code: u8) -> bool {
    matches!(code, 0xE0..=0xE7)
}

/// Converts a modifier HID code into its bitmask position.
pub fn hid_to_modifier_bit(code: u8) -> u8 {
    match code {
        0xE0 => 0x01, // Left Control
        0xE1 => 0x02, // Left Shift
        0xE2 => 0x04, // Left Alt
        0xE3 => 0x08, // Left GUI / Win
        0xE4 => 0x10, // Right Control
        0xE5 => 0x20, // Right Shift
        0xE6 => 0x40, // Right Alt
        0xE7 => 0x80, // Right GUI / Win
        _ => 0x00,
    }
}
