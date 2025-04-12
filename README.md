# macOS UI + Network Watcher

This project demonstrates how to track UI events and network activity on macOS in real time. It includes:

- A **SwiftUI-based macOS app** that simulates user interaction by performing HTTP requests on button clicks.
- A **Rust CLI tool** that:
  - Hooks into mouse and keyboard input using `CGEventTap`
  - Uses the macOS Accessibility API to resolve the UI element under the cursor
  - Logs UI metadata including accessibility label, role, and PID
  - Observes per-process network traffic using `nettop`

This is the macOS counterpart to the [Windows UI + Network Watcher](#), built to give developers deep visibility into app behavior.

---

## 📁 Project Structure

```
.
├── example-mac-app/         # SwiftUI frontend (Xcode project)
├── macos-watcher/           # Rust CLI that logs UI + network activity
```

---

## ✨ Features

- System-wide mouse click and key press detection
- Element inspection via Accessibility APIs
- Logs:
  - App name and PID
  - Button label and accessibility ID
  - UI element role
- Real-time network deltas (↑ bytes sent, ↓ bytes received) using `nettop`

---

## 🚀 Getting Started

### 1. Xcode Project Configuration

To use this project, create `Config/Debug.xcconfig` and `Config/Release.xcconfig` files and populate it with the following values:

```xcconfig
DEVELOPMENT_TEAM = YOUR_TEAM_ID_HERE
```


### 1. Run the SwiftUI App

Open the Xcode project:

```bash
open example-mac-app/example-mac-app.xcodeproj
```

Run the app. It has three buttons (A, B, C) that make test HTTP requests using `URLSession`.

---

### 2. Build and Run the Watcher

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then build and run:

```bash
cd macos-watcher
cargo build --release
./target/release/macos-watcher
```

> ✅ You will be prompted to grant **Accessibility permissions**.  
> Go to: **System Settings → Privacy & Security → Accessibility**  
> and enable access for the built binary.

---

## 🧾 Log Output

Logs are written to:

```
~/macos_watcher.log
```

Example output:

```text
[INFO] Button Clicked: App='example-mac-app', PID=47727, ID='ButtonA', Label='Button A'
📡 example-mac-app.47727 ↑ 6092 B ↓ 0 B (Δ ↑ 6092 ↓ 0)
```

You’ll also see key presses and other clickable UI elements logged with detailed context.

---

## 🛑 How to Stop

This tool runs in the foreground. Press `Ctrl+C` to exit.

---

## 🧠 Use Cases

- See exactly what happens when a UI element is clicked
- Trace which app is making a suspicious network request
- Reverse engineer app behavior without modifying code
- Learn how to use native macOS APIs from Rust

---

## 🧭 Cross-Platform Support

Looking for Windows?  
Check out the [Windows UI + Network Watcher](#)

This macOS version mirrors the Windows tool's architecture and goals:  
Trigger simulated user events in the GUI, observe and attribute network traffic at runtime.