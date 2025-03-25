# foam_shot

A lightweight screenshot utility based on the Wayland screen capture protocol (`wlroots` extension protocol).

> ⚠️ **Development Status**: Currently in early experimental phase. APIs and features may undergo significant changes.

---

## Features

- 🔍 **Area Selection Capture**: Interactive screen region selection with PNG output

---

## Usage

```
cli Options:
      --show-cursor                show cursor when screen freeze, default to false
  -o, --output-path <OUTPUT_PATH>  output path, default to xdg user picture dir, supports format specifiers like %Y, %m, %d, %H, %M, %S
      --no-quickshot               disable quickshot, default to true
      --no-copy                    
  -h, --help                       Print help
  -V, --version                    Print version
```
* When in quick mode, output and exit directly after selection is completed.
* Edit mode is under development.
* in hyprland,like `bind = $mainMod, A, exec, $HOME/.cargo/bin/foam_shot -o /home/username/Pictures/Screenshots/foam_shot-%Y-%m-%d_%H-%M-%S.png
`

---

## Known issues

1. If you enter freeze mode and click directly without moving the mouse, you cannot take a screenshot(In subsequent development, a new method for obtaining mouse coordinates will be used).

---
---
## Roadmap
- [ ] **Multi-monitor coordinated capture**
- [ ] **Cross-compositor compatibility layer**
- [x] **CLI parameters**
- [ ] **Quick-edit mode**
- [ ] **Multi-modal operations**
