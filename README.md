# AULA F108 Pro Mechanical Keyboard - Linux Configuration

[![Português](https://img.shields.io/badge/Portugu%C3%AAs-README.pt.md-green)](README.pt.md)
[![Native Linux GUI](https://img.shields.io/badge/GUI-f108--pro--gui-purple.svg)](f108-pro-gui)
[![Rust Driver](https://img.shields.io/badge/Driver-Rust%20CLI-orange.svg)](f108-pro-rust)

> 🚀 **NEW: Native Linux Suite (No Wine required!)**
> - **[AULA F108 Pro Native GUI (`f108-pro-gui`)](f108-pro-gui)**: Modern WebKitGTK interface, interactive 104-key canvas, real-time collapsible CLI composer, 20 lighting modes, per-key RGB, and TFT LCD screen manager.
> - **[AULA F108 Pro Rust Driver (`f108-pro-rust`)](f108-pro-rust)**: High-performance, zero-overhead standalone Linux CLI driver with USB HID & TFT LCD support.
>
> *(The legacy Wine setup guide is preserved below for reference).*

This repository provides tools, drivers, and guides to configure the **AULA F108 Pro** mechanical keyboard on Linux natively or via Wine.

The AULA F108 Pro keyboard is usually identified by the system as `Bus XXX Device YYY: ID 0c45:800a Microdia Vivitar Vivicam3350B` in `lsusb`.

**Trivia:** The manufacturer *Sonix/Microdia* (`Vendor ID: 0c45`) is the main chip provider for the vast majority of modern mechanical keyboards (including AULA, Redragon, Royal Kludge). The name "Vivitar Vivicam" is merely a naming conflict in the Linux USB database, which thinks this ID belongs to an old digital camera, but it's actually your keyboard's controller.

Here is the exact guide to get it working, using the confirmed ID `0c45:800a`.

---

## Step Zero: How to find the exact ID of any keyboard using an On-Screen Keyboard

If you are configuring another keyboard or on another machine and need to confirm its exact ID, the most reliable method is to compare the connected USB devices list before and after removing the cable. Since you won't have a keyboard to press the "Enter" key, the trick is to use an on-screen keyboard:

1. Open a virtual keyboard on your Linux (on Mint/Ubuntu, you can search for **Onboard** in the start menu, or enable the **On-Screen Keyboard** in Accessibility Settings).
2. Open the terminal and type `lsusb` (but don't press Enter yet).
3. With the virtual keyboard already open on the screen, use the mouse and click the **Enter / Return** button on the virtual keyboard. Save or note down the result that appears in the terminal.
4. Type `lsusb` again in the terminal (don't press Enter).
5. Disconnect your keyboard's cable.
6. Click the **Enter** button on the virtual keyboard again with your mouse.
7. Compare the two lists! The entire line that **disappeared** in the second listing is exactly your keyboard. Save the ID (the numbers in the format `xxxx:yyyy`, for example `0c45:800a`).

## Step 0.5: Install the Software on Wine

Before dealing with advanced system permissions, make sure the official software is installed on your Linux via Wine:

1. Download the official installer from the [AULA Gaming Download Page](https://www.aulagaming.com/pages/download?srsltid=AU7gw4X016m2J_EuRJDXVzFc_UaQqpcjpYPqa6QOKvJNLoxv8DQ099PS) or use the installer provided in this repository (if available).
2. Extract the installer from inside the compressed `.zip` or `.rar` folder.
3. Right-click the `.exe` installer and choose **"Open with Wine"** (or something like "Wine Windows Program Loader"). 
   
   **Terminal Alternative (Command Line):**
   If you prefer installing via the terminal, just open your folder, extract the file and run via Wine:
   ```bash
   wine InstallerName.exe
   ```

   *(Attention: If during execution Wine asks for permission to download or install extra packages like Mono or Gecko, you can accept all of them).*
4. Just advance through the installation (Next button) until finished, exactly as you would on Windows.

## Step 1: Create the `udev` permission rule

Open your terminal and run the `nano` editor as administrator to create a dedicated file for your keyboard:

```bash
sudo nano /etc/udev/rules.d/99-aula-f108-pro.rules
```

Paste the following content into the file:

```udev
# udev rule to grant access to the AULA F108 Pro software via Wine (USB Cable)
SUBSYSTEM=="usb", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", ENV{ID_GPHOTO2}="" 
KERNEL=="hidraw*", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="800a", MODE="0666", ENV{ID_GPHOTO2}=""

# udev rule to grant access to the software via 2.4G wireless Dongle
SUBSYSTEM=="usb", ATTRS{idVendor}=="05ac", ATTRS{idProduct}=="024f", MODE="0666"
KERNEL=="hidraw*", ATTRS{idVendor}=="05ac", ATTRS{idProduct}=="024f", MODE="0666"
```

> **Understanding the generated rule:**
> - `SUBSYSTEM=="usb"` and `KERNEL=="hidraw*"`: Tells Linux that this rule applies to the USB connection and the "raw" communication interface (raw HID) of the device. The software needs this direct access to be able to record macros and modify the RGB colors.
> - `ATTRS{idVendor}=="0c45"` and `ATTRS{idProduct}=="800a"`: This is where we apply the data discovered with the `lsusb` command. This serves as a "filter" to ensure the special permission is given **only** to your AULA keyboard, keeping the security of the rest of your USB devices intact.
> - `MODE="0666"`: Grants read and write (Read/Write) permissions for the device. Since Wine runs the keyboard software using your regular user account (and not as administrator/root), without `0666` the program would be blocked by Linux from trying to change the hardware.
> - `ENV{ID_GPHOTO2}=""`: Removes the "digital camera" classification that the system was erroneously imposing on this device. Without this, some Linux and Wine USB managers block the HID communication.

To save and exit `nano`:
1. Press `Ctrl+O` and hit `Enter` to save.
2. Press `Ctrl+X` to close.

## Step 2: Reload the rules in the system

For your Linux machine to read this file right now without having to restart the computer, run:

```bash
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Step 2.5: Configure the Wine Registry

By default, Wine might block input devices to avoid conflicts with the Linux system. To force Wine to read Raw USB Devices (raw HID), apply these configurations in the registry by running the commands below in the terminal:

```bash
wine reg add "HKLM\System\CurrentControlSet\Services\WineBus" /v "Enable SDL" /t REG_DWORD /d 0 /f
wine reg add "HKLM\System\CurrentControlSet\Services\WineBus" /v "DisableInput" /t REG_DWORD /d 0 /f
```
After that, restart the Wine services:
```bash
wineserver -k
```

## Step 3: Run the AULA Software

With the official software already installed through Wine, you must start it. The exact command will depend on where Wine placed the folder, but it is usually the following:

```bash
wine ~/.wine/drive_c/ProgramFilesx86/AULA_F108Pro/DeviceDriver.exe 
```
*(If a shortcut was created on your Desktop, you can also simply double-click it if Wine is configured for that).*

---

## Bonus: How to fix the keyboard name in `lsusb`

If you want to fix the Linux "aesthetic error" and make the `lsusb` command correctly display the name "AULA F108 Pro" instead of "Vivitar Vivicam3350B", you can edit your system's USB names database.

1. Open the USB database file (`usb.ids`) as administrator:
```bash
sudo nano /usr/share/hwdata/usb.ids
```
*(Note: depending on your Linux distribution, the file might be located at `/usr/share/misc/usb.ids`)*

2. In nano, press `Ctrl+W` to open the search, type `800a` and press `Enter`.
3. It will jump directly to the following line:
   `	800a  Vivitar Vivicam3350B`
4. Just delete the camera name and type your keyboard's name, leaving the line like this:
   `	800a  AULA F108 Pro`
5. Save and exit nano (`Ctrl+O`, `Enter`, `Ctrl+X`).

Done! The next time you run the `lsusb` command, your list will proudly display the correct name of your keyboard!
