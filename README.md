# XP Enhance 0.5.0

XP Enhance is a Windows desktop companion for X-Plane 12. It provides an iOS 27-inspired liquid-glass control surface for supported simulator visual settings, third-party add-on discovery, ReShade preset management, system telemetry and an optional in-simulator FPS overlay.

## Included in 0.5.0

- Redesigned translucent, layered interface with rounded glass panels, ambient lighting, compact navigation and responsive controls.
- Performance, Balanced, Realistic and Cinematic visual profiles, a master intensity control and feature switches.
- Optional FlyWithLua NG visual bridge using the bundled MIT-licensed RXP XP12 Enhancer script. Supported controls include exposure/tone mapping, global illumination, night lighting, extended shadow distance, SSAO, water-wave parameters, rain effects, bloom-artifact reduction and FSR sharpening.
- Optional XP Enhance FPS overlay installed into the FlyWithLua Scripts folder. The counter reads `sim/operation/misc/frame_rate_period` and draws the live FPS value inside X-Plane. Enable it in Settings and choose a screen corner. Reload FlyWithLua scripts after installation or changes.
- Live METAR briefing lookup by ICAO station through the NOAA AviationWeather API (read-only briefing).
- Direct X-Plane weather preset controls through the simulator local Web API: Clear, VFR Few/Scattered/Broken/Marginal, IFR Non-precision/Precision, Convective and Large-cell Storms, plus a real-weather mode request.
- CPU and system-memory telemetry. NVIDIA GPU utilization, temperature and VRAM are shown when `nvidia-smi` is available. AMD and Intel GPU telemetry are not currently exposed.
- Detection and publisher links for Active Sky XP 12, VisualXP, X-Clouds Enhancer, Ultimate Water Enhancement, Immersive Lights and LuminaXP. XP Enhance does not redistribute their installers.
- ReShade INI import, deployment, runtime detection and configured shader search-path checks.
- Local settings persistence, X-Plane add-on inventory scanning, backups before integration-file replacement, and GitHub release update checking.

## FPS overlay setup

1. Install FlyWithLua NG for X-Plane 12 from its official distribution.
2. Set and scan your X-Plane 12 installation folder in XP Enhance.
3. Open Performance and select **Install FPS overlay** (or install the full visual bridge from Integrations).
4. Open Settings, enable **Show FPS in X-Plane 12**, and choose a screen corner.
5. In X-Plane, reload FlyWithLua scripts or restart the simulator.

The overlay is drawn by FlyWithLua inside the simulator. It is not a desktop always-on-top window. It uses the simulator frame-time dataref and displays the reciprocal as FPS.

## Weather and third-party integrations

The METAR panel retrieves the latest available station report from AviationWeather.gov. It does not write weather data into X-Plane. Active Sky XP 12 advertises a third-party API, but XP Enhance 0.5.0 only detects the installation and links to the publisher; direct Active Sky API control is not enabled yet.

Third-party add-on cards provide installation detection and official publisher/download links. They do not install paid or third-party files automatically. Check each add-on's current X-Plane version, GPU, Vulkan, VR and multi-monitor compatibility before installing it. Avoid running multiple plugins that modify the same private rendering datarefs without checking compatibility.

## ReShade

XP Enhance imports ReShade `.ini` files and copies them into the X-Plane directory. The dependency check verifies configured effect and texture search directories. It cannot guarantee that every referenced shader exists or that every effect is compatible with the installed ReShade/Vulkan runtime. Install ReShade and shader packs separately from their respective publishers.

## Performance telemetry

CPU and memory values are system-wide, not X-Plane-process-only. NVIDIA GPU values require the `nvidia-smi` executable supplied with a compatible NVIDIA driver. If it is unavailable, GPU values are shown as unavailable. This release does not provide AMD/Intel GPU telemetry or simulator frametime graphs.

## Visual bridge compatibility

The bundled RXP enhancer is a community MIT-licensed Lua script that uses X-Plane private rendering datarefs. Its upstream author reports testing against X-Plane 12.4.0. Private datarefs may change in later simulator versions. Keep backups and verify the exact simulator build. The bridge is not a custom renderer and does not replace X-Plane's cloud, weather, water or lighting engines.

## Update mechanism

The app checks the latest stable GitHub release at startup. When a newer release is found, it displays an update banner. On Windows, the app downloads the installer, checks its SHA-256 digest against GitHub release metadata, and launches the installer. This is an integrity check against GitHub's release metadata, not a code-signing certificate or protection against a compromised release account.

## Not included / limitations

- No custom cloud-rendering engine, volumetric cloud shader replacement or cloud-texture generator.
- No new water-reflection rendering pass beyond controls exposed by the simulator bridge or external add-ons.
- No native frame-generation engine; LuminaXP owns its frame-generation controls inside X-Plane.
- No direct Active Sky XP 12 API control in this release. Weather preset writes are limited to X-Plane’s documented local Web API and supported regional preset dataref.
- No AMD/Intel GPU telemetry, and no per-process X-Plane CPU/GPU breakdown.
- No guarantee that every ReShade preset or third-party plugin will work with every X-Plane build.

These limitations are explicit because a desktop companion cannot safely replace X-Plane's renderer or take ownership of proprietary third-party plugin APIs without their documented interfaces and compatibility testing.

## Build

The repository uses Tauri 2, Rust, TypeScript and Vite. GitHub Actions builds the Windows NSIS installer on `windows-latest`.
