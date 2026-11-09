# XP Enhance graphics-engine development plan

## Goal

Build XP Enhance-owned rendering tools and visual effects for X-Plane 12. The desktop application owns profiles and controls. Any in-simulator component must be an independently authored XP Enhance module, with clear backups, reversible installation and a safe disable path.

## What can be built independently

1. **Profile and effect system** — a versioned preset format, colour-grading curves, exposure controls, sharpening, tone mapping and user-adjustable effects. The desktop UI can author and validate these profiles.
2. **Supported simulator bridge** — use the X-Plane Plugin SDK for supported data access, commands and plugin UI. This can expose telemetry and controls without patching simulator binaries.
3. **Own rendering experiments** — evaluate an explicitly unsupported Vulkan capture/injection prototype in a separate test harness before considering any X-Plane integration. Do not deploy it into users' simulators until stability, frame pacing, device loss, VR, multi-monitor, driver compatibility and uninstall/rollback are tested.
4. **Native DLSS 5 integration** — only after NVIDIA grants access to the DLSS 5 Neural Rendering SDK/runtime and the required integration contract. The implementation needs engine scene inputs such as motion vectors and masks, not just a screenshot or ReShade-style colour pass.

## Why the X-Plane integration is blocked

Laminar Research states that X-Plane 12 does not provide a stable supported interface for third-party plugins to hook its drawing pipeline or modify its shaders. Vulkan-layer injection may be technically possible, but Laminar explicitly warns that internal command streams and shaders can change between updates and offers no compatibility guarantee for this approach. A plugin using the supported XPLM drawing APIs is not equivalent to a full-scene post-processing hook.

NVIDIA's DLSS 5 is 3D-guided neural rendering, not a generic sharpening filter. NVIDIA describes it as consuming game-engine rendering inputs including motion vectors and offering scene/object-aware controls. NVIDIA currently lists GeForce RTX 50 Series support. NVIDIA developer forum staff have said the DLSS 5 SDK is not available for public access and that current solutions are through developer-partner cooperation.

## OptiShade comparison

OptiShade is a useful product reference for the workflow—profiles, a manager, in-game controls and optional neural-rendering setup. XP Enhance will not copy its source or bundle its ReShade/OptiScaler components. A similar user experience can be built in-house, but that does not create access to NVIDIA's neural model or a supported X-Plane renderer hook.

## Next engineering gates

- Complete the desktop profile format and effect editor.
- Build a separate XP Enhance-owned Vulkan proof-of-concept against a small test application, not X-Plane.
- Verify frame pacing and safe recovery before any simulator test.
- Seek the official NVIDIA DLSS 5 SDK access route and document required input buffers and licensing.
- Only claim DLSS 5 support after the official runtime is actually executing in X-Plane and verified with captured output.

## References

- X-Plane local Web API: https://developer.x-plane.com/article/x-plane-web-api/
- X-Plane plugin development: https://developer.x-plane.com/article/developing-plugins/
- Laminar Research on engine hooking: https://developer.x-plane.com/2026/10/on-hacking-our-engine-please-dont/
- NVIDIA DLSS 5 technical overview: https://www.nvidia.com/en-us/geforce/news/dlss-5-3d-guided-neural-rendering/
- NVIDIA developer SDK access statement: https://forums.developer.nvidia.com/t/official-dlss-5-neural-rendering-plugin-access-for-unreal-engine-5-8-2/382731/
- OptiShade project overview: https://github.com/GamingWithGravy/OptiShade


## XP Enhance 0.6.2 graphics-manager implementation

The desktop manager now has an OptiShade-style setup workflow for X-Plane's Vulkan path. It launches a ReShade setup executable already present in the user's Downloads folder, or opens the official ReShade download page. The user selects their X-Plane.exe and Vulkan in the official setup tool; XP Enhance does not redistribute ReShade binaries or shader repositories.

The manager detects a game-local ReShade configuration and shader folder plus a system-wide ReShade Vulkan layer manifest. It installs an XP Enhance-authored ReShade FX colour-grade shader (exposure, contrast, saturation and warmth), backing up an existing copy before replacement. It can import a user-selected ReShade .ini preset into an XPEnhance-Presets folder, preserving a same-name previous preset as a backup. Imported presets are selected from ReShade's in-game overlay.

This is an actual ReShade post-processing integration path, not DLSS 5. The ReShade setup remains an external official installer, and X-Plane/Vulkan compatibility must be verified on the user's machine. XP Enhance does not claim neural rendering, upscaling or frame generation from the shader effects.
