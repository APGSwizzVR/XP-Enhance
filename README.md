# XP Enhance

XP Enhance is a Windows desktop control centre for X-Plane 12 visual profiles and simulator content.

## Current application
- Aviation-inspired flight-deck interface and instrument-panel visual language.
- Performance, Balanced, Realistic and Cinematic profiles.
- Ten visual-system profile switches and a 0–100 realism control.
- Persistent settings stored in the operating system application configuration directory.
- ReShade INI import, local storage and configuration preview.
- X-Plane directory auto-detection for common Windows locations and manual path configuration.
- Simulator folder validation and inventory scans for plugins, Custom Scenery and aircraft.
- Windows NSIS installer built by GitHub Actions.

## Integration status
This release is the desktop control centre and profile manager. It does not inject shaders or modify X-Plane's renderer. Weather choices are saved preferences, not live weather overrides. Add-on inventory is read-only. Live FPS/GPU/CPU telemetry requires a simulator-side integration.

A genuine in-simulator effects layer must be built and tested against the Laminar Research X-Plane Plugin SDK and a renderer-compatible graphics implementation. The profile switches are ready to be connected to that layer; they are not represented as active rendering effects until integration exists.

## Build
Requirements: Node.js 22+, Rust stable and Windows build tools.

Run npm install, then npm run tauri -- dev. Build the installer with npm run tauri -- build --bundles nsis.

Pushes to main run the Windows GitHub Actions workflow and publish a prerelease installer.