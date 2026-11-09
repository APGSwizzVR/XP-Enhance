# XP Enhance

XP Enhance is a Windows desktop control centre for X-Plane 12 visual enhancement profiles.

## Current foundation

- Rust application backend with Tauri 2.
- Desktop interface built with TypeScript, Vite and the Windows WebView2 runtime.
- Local settings persistence in the user's application configuration directory.
- Visual feature toggles, realism intensity and selectable presets.
- ReShade INI import and local storage.
- X-Plane installation path setting.
- GitHub Actions builds a Windows NSIS installer and publishes it as a GitHub prerelease.

## Development

Requirements: Node.js 22, Rust stable, and the Windows C++ build tools / WebView2 prerequisites for Windows development.

```sh
npm install
npm run tauri -- dev
```

Build the Windows installer:

```sh
npm run tauri -- build --bundles nsis
```

## Integration status

The current release is the desktop foundation. Settings and profile storage work locally, but the visual options are not yet connected to X-Plane's rendering pipeline. ReShade INI files are imported and stored; shader injection and automatic add-on installation are future integration work.
