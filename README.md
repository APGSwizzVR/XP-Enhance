# XP Enhance 0.6.0

XP Enhance is a Windows desktop workspace for configuring and saving a personal X-Plane 12 setup. Version 0.6.0 replaces the previous add-on-centred dashboard and legacy integration backend with a graphite interface and a local-first application core.

## Included

- A graphite #363636 application theme with translucent glass surfaces, restrained motion and system-native typography.
- Visual Studio for organising lighting, world/material and image-quality preferences.
- A dedicated DLSS 5 profile workspace with NVIDIA GPU-family selection, Natural/Cinematic/Realism styles, and detailed image, temporal, motion and scene controls.
- Persistent DLSS profile settings, including frame-generation and ray-reconstruction preferences.
- A local Atmosphere profile editor with cloud, haze, humidity, visibility, sunlight, precipitation and turbulence preferences.
- CPU and memory telemetry, plus graphics-device identification through Windows system inventory. GPU load, temperature and VRAM sensors are not currently exposed.
- X-Plane folder detection and read-only inventory of aircraft, scenery and plugin folders.
- Local profile persistence.

## Live performance telemetry

The Performance page samples automatically once per second. It shows system CPU and memory, available memory, swap, CPU core/thread counts and frequency, running process count, X-Plane process CPU/memory, and the graphics device identified by Windows. The current build identifies the graphics device but does not yet expose reliable GPU load, temperature or VRAM sensor readings; those fields remain unavailable rather than fabricated.

X-Plane FPS and frame-time readings are read from the simulator's local Web API. This requires X-Plane 12.1.1 or newer, a loaded flight, and incoming traffic enabled in Settings → Network. The default API port is 8086. The separate in-simulator FPS switch invokes X-Plane's own built-in FPS display command; it is independent of the desktop Performance tab.

## Rendering status and limitations

The DLSS 5 tab is a configuration workspace only. It does not contain NVIDIA DLSS 5 runtime code, run a neural rendering model, or enable DLSS 5 inside X-Plane. NVIDIA currently lists GeForce RTX 50 Series support. NVIDIA developer forum staff have stated that the DLSS 5 SDK is not yet publicly available and access is currently through developer-partner cooperation. Genuine X-Plane support requires NVIDIA SDK access and direct integration into the simulator's rendering pipeline. A desktop settings panel cannot create that integration by itself.

See NVIDIA's DLSS 5 FAQ (https://www.nvidia.com/en-us/geforce/forums/legacy-products/12/583738/dlss-5-faq/) and the NVIDIA Developer Forum SDK access discussion (https://forums.developer.nvidia.com/t/official-dlss-5-neural-rendering-plugin-access-for-unreal-engine-5-8-2/382731/).

The selected GPU family, visual style and sliders are saved as an XP Enhance profile; they do not currently change rendered frames. Visual Studio and Atmosphere also save local profile values. They are not yet a replacement rendering, cloud, water or weather engine and do not inject their values into X-Plane.

## Independent runtime

The 0.6.0 application no longer bundles or calls FlyWithLua, ReShade, LuminaXP, Active Sky, VisualXP, X-Clouds, UWE or other third-party simulator products. Its native backend is limited to local settings, X-Plane folder inventory and Windows system telemetry. Rust crates and Tauri/npm packages remain ordinary build dependencies under their own licences.

The Atmosphere editor is the start of XP Enhance's own weather work, not a completed weather simulation. The native renderer and weather engine still need to be developed and tested against X-Plane's supported SDK.

## Build

The project uses Tauri 2, Rust, TypeScript and Vite. GitHub Actions builds the Windows x64 NSIS installer.

## Copyright

See LICENSE, COPYRIGHT.md and THIRD-PARTY-NOTICES.md.
