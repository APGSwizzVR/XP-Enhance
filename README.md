# XP Enhance 0.4.0

XP Enhance is a Windows desktop companion for X-Plane 12. This update replaces the previous aviation-instrument UI with a dark iOS-inspired liquid-glass interface and adds a real, optional FlyWithLua bridge for supported visual settings.

## Included in 0.4.0

- Complete liquid-glass interface redesign and iOS-style 51×31 switches.
- Performance, Balanced, Realistic and Cinematic profiles.
- Persistent local settings and a 0–100 visual-intensity control.
- Optional installation of the MIT-licensed RXP X-Plane 12 Enhancer Lua script into an existing FlyWithLua NG installation.
- Simulator settings synchronization to the enhancer's `rxp-xp12-enhancer.ini` file.
- Supported bridge controls: exposure/tone mapping, global illumination, night lighting, extended shadow distance, SSAO, water-wave parameters, rain effects, bloom-artifact reduction and FSR sharpening.
- Safe backup of existing script, INI and ReShade preset files before replacement.
- ReShade preset import and copy to the X-Plane folder as `XPEnhance-ReShade.ini`.
- Detection of FlyWithLua, ReShade and LuminaXP folders/configuration.
- Read-only inventory scans for plugins, scenery and aircraft.

## Installing the X-Plane visual bridge

1. Install FlyWithLua NG for X-Plane 12 using its official distribution.
2. Close X-Plane 12.
3. In XP Enhance, set the X-Plane folder and open **Integrations**.
4. Select **Install visual bridge**. XP Enhance copies the Lua script, license and generated settings INI into `Resources/plugins/FlyWithLua/Scripts`.
5. Start X-Plane and reload FlyWithLua scripts (or restart the simulator) to apply the settings.
6. Changes made in XP Enhance update the INI file. Reload FlyWithLua scripts in X-Plane to apply changes to an already running session.

The included script is from `SmusiJarviss/RXP-XP12-Enhancer`, which identifies the script as an MIT-licensed modification of the Reality XP enhancer. The license is included beside the script. The upstream author reports testing against X-Plane 12.4.0 and warns that newer simulator versions may behave differently. It uses private X-Plane rendering datarefs, which Laminar Research may change. Use at your own risk and keep backups.

## ReShade

XP Enhance imports a ReShade `.ini` preset and copies it to the X-Plane directory. Install ReShade separately from [reshade.me](https://www.reshade.me/) and select the Vulkan API for X-Plane 12. The app does not redistribute ReShade binaries or third-party shader packs. Presets may reference shader/effect packages, textures, paths, and ReShade versions that must be installed separately; therefore, no application can guarantee that every arbitrary preset works unchanged.

## Frame generation

Frame generation is not implemented inside XP Enhance. The Performance page can detect a LuminaXP folder and links to the community plugin page. LuminaXP is a separate beta plugin; compatibility depends on the simulator version, GPU, driver, and plugin build. XP Enhance does not bundle or redistribute it.

## Not implemented in this release

- A replacement weather engine or REX Atmos Core-equivalent cloud engine.
- New cloud rendering, cloud texture generation, or live weather injection.
- New water-reflection rendering passes beyond the parameters exposed by the bundled script.
- A custom in-game shader renderer or universal shader injector beyond ReShade's own Vulkan runtime.
- Native frame generation inside XP Enhance.
- Guaranteed support for every ReShade preset or shader dependency.
- Live FPS/GPU/CPU telemetry.

These are deliberately shown as unconnected/external capabilities rather than presented as working features.

## Build

The repository uses Tauri 2, Rust, TypeScript and Vite. GitHub Actions builds the Windows NSIS installer on `windows-latest`.
