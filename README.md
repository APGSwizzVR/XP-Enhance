# XP Enhance 0.6.0

XP Enhance is a Windows desktop workspace for configuring and saving a personal X-Plane 12 visual profile. The 0.6.0 release replaces the previous add-on-centred dashboard with a graphite interface, translucent glass surfaces and a local-first settings model.

## What is in 0.6.0

- A redesigned graphite `#363636` application theme with translucent panels, subtle reflections, restrained animation and system-native typography.
- A new Visual Studio for organising visual preferences into lighting, world/material and image-quality groups.
- A dedicated DLSS 5 profile workspace with NVIDIA GPU-family selection, Natural/Cinematic/Realism looks, effect-strength controls and detailed image, temporal, motion and scene sliders.
- Persistent local DLSS profile settings, including frame-generation and ray-reconstruction preferences. These are saved configuration values, not active rendering features.
- A local Atmosphere profile editor with cloud, haze, humidity, visibility, sunlight, precipitation and turbulence preferences.
- System CPU, memory and available NVIDIA GPU telemetry.
- X-Plane folder detection and read-only inventory of aircraft, scenery and plugin folders.
- Local profile persistence.

## Important rendering limitations

The DLSS 5 tab is a profile/configuration workspace only. This release does **not** contain NVIDIA DLSS 5 runtime code, does not run a neural rendering model, and does not enable DLSS 5 inside X-Plane. Genuine DLSS 5 support requires an official compatible NVIDIA runtime/SDK and direct integration into the simulator's rendering pipeline. A desktop settings panel cannot create that integration by itself. The selected GPU family, visual style and sliders are saved for the future native renderer work; they do not currently change rendered frames.

Likewise, Visual Studio and Atmosphere currently save local profile values. They are not yet a replacement rendering, cloud, water or weather engine and do not inject their values into the simulator. XP Enhance does not claim otherwise.

## Local-first approach

The redesigned interface no longer promotes or requires Active Sky, VisualXP, X-Clouds, UWE, LuminaXP or ReShade. The profile editor and its settings are owned by XP Enhance. X-Plane inventory scanning is read-only.

The existing Rust backend still contains legacy integration code and bundled third-party resource notices for backwards compatibility; the new interface does not expose those legacy integrations as required services. Their code and licensing remain separate from XP Enhance-owned code.

## Build

The project uses Tauri 2, Rust, TypeScript and Vite. GitHub Actions builds the Windows x64 NSIS installer.

```bash
npm install
npm run build
npm run tauri build
```

## Copyright

See `LICENSE`, `COPYRIGHT.md` and `THIRD-PARTY-NOTICES.md`.
