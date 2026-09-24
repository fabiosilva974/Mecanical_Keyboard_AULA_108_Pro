# AULA F108 Pro GUI - TFT LCD Display & Clock Sync Protocol

## 1. Display Hardware Specifications
- **Display Resolution**: 240 pixels (width) x 135 pixels (height).
- **Color Depth**: RGB565 (16 bits per pixel: 5 bits Red, 6 bits Green, 5 bits Blue).
- **USB Interface**: USB Interface 2 (Interrupt OUT Endpoint 3, Interrupt IN Endpoint 4 for acknowledgments).
- **Transmission Frame Size**: 4096-byte pages streamed sequentially.

## 2. Hardware Real-Time Clock (RTC) Sync
The keyboard features a built-in calendar and clock shown on the TFT standby screen.
- Command opcode: `0x04 0x28`
- Data payload: `[0x5A, year_hi, year_lo, month, day, hour, minute, second]`
- CLI invocation:
  ```bash
  f108-pro clock
  ```

## 3. Image & GIF Flashing Pipeline
1. **User Selects Image**: Any PNG, JPEG, BMP, or animated GIF.
2. **Pillow Preprocessing**:
   - Resized to exact 240x135 aspect ratio using Lanczos bicubic filtering.
   - Preserves animation frame delay.
3. **mkimage Binary Compilation**:
   - Converts frames to RGB565 binary format (`.bin`).
4. **Driver Streaming**:
   - `f108-pro lcd <file.bin>` sends header packet (`0x04 0x72`) and streams 4KB chunks with device ACK verification.
