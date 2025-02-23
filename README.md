# Smart Media Key

A utility that simulates AirPods-style media controls using keyboard tap patterns. Originally created because while Wooting keyboards have features like snappy tap and pressure-based inputs, they don't natively support tap pattern detection (sometimes called happy tap in other software).

## What it does

The utility detects different tap patterns on a key and converts them into actions. By default, it mimics AirPods behavior using the F13 key:
- Single tap -> Play/Pause
- Double tap -> Next Track
- Triple tap -> Previous Track
- Long press -> F14 (usable for voice assistant/speech-to-text/etc)

While the default mappings mirror AirPods, the underlying system can handle any combination of tap patterns to trigger any key press. The groundwork exists for configuring different keys and actions through JSON.

## Features

### Implemented
- [x] Pattern detection system
- [x] Wooting SDK integration
- [x] Basic system tray presence
- [x] AirPods-style media controls
- [x] Cross-platform tray support
- [x] GitHub actions to release version (temporary)

### Possibly will do in the future:
- [ ] Standard keyboard support
- [ ] JSON-based key mapping configuration
- [ ] System tray configuration interface
- [ ] Full cross-platform feature support
- [ ] Code organization/modularization
- [ ] Proper icon for the app
- [ ] Improved release GitHub action for platforms etc.
- [ ] Test coverage
