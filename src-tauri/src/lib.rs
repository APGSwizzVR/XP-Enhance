use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command, time::{Duration, SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};
use sysinfo::System;

const RXP_SCRIPT: &str = include_str!("../resources/rxp-xp12-enhancer.lua");
const RXP_LICENSE: &str = include_str!("../resources/RXP-ENHANCER-LICENSE.txt");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct FeatureSettings {
    colour: bool,
    sharpening: bool,
    ambient_occlusion: bool,
    bloom: bool,
    cloud_enhancement: bool,
    anti_aliasing: bool,
    depth_of_field: bool,
    water_reflections: bool,
    night_lighting: bool,
    terrain_detail: bool,
}
impl Default for FeatureSettings {
    fn default() -> Self {
        Self { colour: true, sharpening: true, ambient_occlusion: true, bloom: false, cloud_enhancement: true, anti_aliasing: true, depth_of_field: false, water_reflections: true, night_lighting: true, terrain_detail: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    preset: String,
    realism: u8,
    xplane_path: Option<String>,
    reshade_ini: Option<String>,
    features: FeatureSettings,
    weather_profile: String,
    scenery_profile: String,
    telemetry_enabled: bool,
    fps_overlay_enabled: bool,
    fps_overlay_position: String,
    xplane_api_port: u16,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self { preset: "Balanced".into(), realism: 68, xplane_path: None, reshade_ini: None, features: FeatureSettings::default(), weather_profile: "Live weather".into(), scenery_profile: "Balanced".into(), telemetry_enabled: true, fps_overlay_enabled: false, fps_overlay_position: "top-right".into(), xplane_api_port: 8086 }
    }
}

#[derive(Serialize)]
struct Addon { name: String, path: String, kind: String }
#[derive(Serialize)]
struct ScanResult { plugins: Vec<Addon>, scenery: Vec<Addon>, aircraft: Vec<Addon>, xplane_found: bool, version: Option<String> }
#[derive(Serialize)]
struct IntegrationStatus {
    flywithlua_installed: bool, visual_script_installed: bool, fps_overlay_installed: bool,
    reshade_detected: bool, lumina_detected: bool, lumina_manager_found: bool, lumina_plugin_binary_found: bool,
    active_sky_detected: bool, visualxp_detected: bool, xclouds_detected: bool, uwe_detected: bool, immersive_lights_detected: bool,
}
#[derive(Serialize)]
struct MetarReport { station: String, observed: Option<String>, temperature_c: Option<String>, dewpoint_c: Option<String>, wind: Option<String>, visibility_sm: Option<String>, altimeter: Option<String>, raw: Option<String> }
#[derive(Serialize)]
struct PerformanceSnapshot { cpu_usage_percent: f32, memory_used_mb: u64, memory_total_mb: u64, gpu_name: Option<String>, gpu_utilization_percent: Option<u32>, gpu_temperature_c: Option<u32>, vram_used_mb: Option<u64>, vram_total_mb: Option<u64> }
#[derive(Serialize)]
struct ReshadeReport { sections: usize, effect_search_paths: Vec<String>, texture_search_paths: Vec<String>, missing_paths: Vec<String> }


fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}
#[tauri::command]
fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_file(&app)?;
    if !path.exists() { return Ok(AppSettings::default()); }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}
#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_file(&app)?;
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
fn valid_xplane(path: &Path) -> bool {
    path.is_dir() && ["X-Plane.exe", "X-Plane 12.exe", "X-Plane-x86_64.exe"].iter().any(|name| path.join(name).is_file())
}
#[tauri::command]
fn detect_xplane() -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("USERPROFILE") {
        let home = PathBuf::from(home);
        candidates.extend([home.join("Desktop/X-Plane 12"), home.join("Documents/X-Plane 12"), home.join("X-Plane 12")]);
    }
    candidates.extend([PathBuf::from("C:/X-Plane 12"), PathBuf::from("D:/X-Plane 12"), PathBuf::from("C:/Program Files/X-Plane 12")]);
    candidates.into_iter().find(|path| valid_xplane(path)).map(|path| path.to_string_lossy().to_string())
}
fn directories(path: &Path, kind: &str) -> Vec<Addon> {
    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let item_path = entry.path();
            if item_path.is_dir() {
                items.push(Addon { name: entry.file_name().to_string_lossy().to_string(), path: item_path.to_string_lossy().to_string(), kind: kind.into() });
            }
        }
    }
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}
#[tauri::command]
fn scan_xplane(path: String) -> Result<ScanResult, String> {
    let root = PathBuf::from(path.trim());
    if !root.exists() || !root.is_dir() { return Err("The configured X-Plane directory does not exist.".into()); }
    if !valid_xplane(&root) { return Ok(ScanResult { plugins: vec![], scenery: vec![], aircraft: vec![], xplane_found: false, version: None }); }
    Ok(ScanResult { plugins: directories(&root.join("Resources/plugins"), "Plugin"), scenery: directories(&root.join("Custom Scenery"), "Scenery"), aircraft: directories(&root.join("Aircraft"), "Aircraft"), xplane_found: true, version: Some("X-Plane 12".into()) })
}
fn flywithlua_dir(root: &Path) -> Option<PathBuf> {
    ["FlyWithLua", "FlyWithLua NG", "FlyWithLua_NG"].iter().map(|name| root.join("Resources/plugins").join(name)).find(|path| path.is_dir())
}
fn reshade_found(root: &Path) -> bool {
    ["ReShade.ini", "ReShade64.dll", "ReShade32.dll"].iter().any(|name| root.join(name).is_file())
        || std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("Vulkan/implicit_layer.d/ReShade64.json").is_file()).unwrap_or(false)
}
#[tauri::command]
fn get_integration_status(path: String) -> Result<IntegrationStatus, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The configured X-Plane folder is not valid.".into()); }
    let fwl = flywithlua_dir(&root);
    let script_installed = fwl.as_ref().map(|dir| dir.join("Scripts/rxp-xp12-enhancer.lua").is_file()).unwrap_or(false);
    let lumina_dir = root.join("Resources/plugins/LuminaXP");
    let lumina_manager_found = lumina_dir.join("LuminaXP.exe").is_file();
    let lumina_plugin_binary_found = [lumina_dir.join("win.xpl"), lumina_dir.join("64/win.xpl"), lumina_dir.join("lin.xpl")].iter().any(|p| p.is_file());
    let lumina = lumina_dir.is_dir() && (lumina_manager_found || lumina_plugin_binary_found);
    let scripts = fwl.as_ref().map(|dir| dir.join("Scripts"));
    let fps_overlay_installed = scripts.as_ref().map(|dir| dir.join("xp-enhance-fps.lua").is_file()).unwrap_or(false);
    let plugins = root.join("Resources/plugins");
    let plugin_names: Vec<String> = fs::read_dir(&plugins).ok().into_iter().flatten().flatten()
        .filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().to_lowercase()).collect();
    let visualxp_detected = plugin_names.iter().any(|n| n.contains("visualxp") || n.contains("visual xp"));
    let xclouds_detected = plugin_names.iter().any(|n| n.contains("x-cloud") || n.contains("xcloud"))
        || scripts.as_ref().map(|d| fs::read_dir(d).ok().into_iter().flatten().flatten().any(|e| e.file_name().to_string_lossy().to_lowercase().contains("x-cloud"))).unwrap_or(false);
    let uwe_detected = plugin_names.iter().any(|n| n.contains("ultimate water") || n == "uwe" || n.contains("water enhancement"));
    let immersive_lights_detected = plugin_names.iter().any(|n| n.contains("immersive light") || n.contains("immersivelight"));
    let mut active_sky_detected = false;
    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(base) = std::env::var_os(key) {
            let base = PathBuf::from(base);
            for candidate in [base.join("HiFi/ASXP12"), base.join("HiFi Technologies/ASXP12"), base.join("Active Sky XP12")] {
                if candidate.exists() { active_sky_detected = true; }
            }
        }
    }
    Ok(IntegrationStatus { flywithlua_installed: fwl.is_some(), visual_script_installed: script_installed, fps_overlay_installed,
        reshade_detected: reshade_found(&root), lumina_detected: lumina, lumina_manager_found, lumina_plugin_binary_found,
        active_sky_detected, visualxp_detected, xclouds_detected, uwe_detected, immersive_lights_detected })
}
fn backup_existing(path: &Path) -> Result<(), String> {
    if !path.exists() { return Ok(()); }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let name = path.file_name().ok_or("Invalid destination filename")?.to_string_lossy();
    let backup = path.with_file_name(format!("{name}.xp-enhance-backup-{stamp}"));
    fs::copy(path, backup).map_err(|e| format!("Could not back up {}: {e}", path.display()))?;
    Ok(())
}
fn bool_text(value: bool) -> &'static str { if value { "true" } else { "false" } }
fn settings_ini(settings: &AppSettings) -> String {
    let f = &settings.features;
    let master = f.colour || f.cloud_enhancement || f.night_lighting || f.terrain_detail || f.ambient_occlusion;
    let realism = settings.realism.min(100);
    let contrast = (realism as f32 * 0.04).min(4.0);
    let sharpen = if f.sharpening { realism } else { 0 };
    format!("[settings]\nauto_show_wnd = false\noverride_exposure = {}\noverride_contrast = {}\noverride_contrast_level = {:.1}\noverride_contrast_range = true\noverride_illumination = {}\noverride_lights = {}\noverride_lights_sheet = 0\noverride_shadows = {}\noverride_rain = {}\noverride_rain_lines = true\noverride_water = {}\noverride_water_style = 2\noverride_ssao = {}\noverride_ssao_cockpit = true\noverride_bloom_effect = {}\noverride_sharpen_level = {}\n", bool_text(master), bool_text(master && realism > 0), contrast, bool_text(f.cloud_enhancement), bool_text(f.night_lighting), bool_text(f.terrain_detail), bool_text(f.anti_aliasing), bool_text(f.water_reflections), bool_text(f.ambient_occlusion), bool_text(f.bloom), sharpen)
}
fn fps_overlay_source(settings: &AppSettings) -> String {
    let position = match settings.fps_overlay_position.as_str() {
        "top-left" | "top-right" | "bottom-left" | "bottom-right" => settings.fps_overlay_position.as_str(),
        _ => "top-right",
    };
    include_str!("../resources/xp-enhance-fps.lua")
        .replace("__FPS_ENABLED__", if settings.fps_overlay_enabled { "true" } else { "false" })
        .replace("__FPS_POSITION__", position)
}
fn write_bridge_settings(root: &Path, settings: &AppSettings) -> Result<String, String> {
    if !valid_xplane(root) { return Err("The X-Plane directory is not valid.".into()); }
    let fwl = flywithlua_dir(root).ok_or("FlyWithLua NG was not found. Install FlyWithLua NG first, then try again.")?;
    let scripts = fwl.join("Scripts");
    fs::create_dir_all(&scripts).map_err(|e| format!("Could not create FlyWithLua Scripts folder: {e}"))?;
    if scripts.join("rxp-xp12-enhancer.lua").is_file() {
        fs::write(scripts.join("rxp-xp12-enhancer.ini"), settings_ini(settings)).map_err(|e| format!("Could not update simulator settings: {e}"))?;
    }
    if scripts.join("xp-enhance-fps.lua").is_file() {
        fs::write(scripts.join("xp-enhance-fps.lua"), fps_overlay_source(settings)).map_err(|e| format!("Could not update FPS overlay: {e}"))?;
    }
    Ok("Simulator settings synchronised. Reload FlyWithLua scripts in X-Plane to apply changes.".into())
}
#[tauri::command]
fn sync_visual_settings(path: String, settings: AppSettings) -> Result<String, String> {
    write_bridge_settings(Path::new(path.trim()), &settings)
}
#[tauri::command]
fn install_visual_integration(path: String, settings: AppSettings) -> Result<String, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The X-Plane directory is not valid. Select the folder containing X-Plane.exe.".into()); }
    let fwl = flywithlua_dir(&root).ok_or("FlyWithLua NG was not found. Install the latest FlyWithLua NG for X-Plane 12 first.")?;
    let scripts = fwl.join("Scripts");
    fs::create_dir_all(&scripts).map_err(|e| format!("Could not create Scripts folder: {e}"))?;
    let script_path = scripts.join("rxp-xp12-enhancer.lua");
    if script_path.exists() && fs::read_to_string(&script_path).map(|s| s != RXP_SCRIPT).unwrap_or(true) { backup_existing(&script_path)?; }
    fs::write(&script_path, RXP_SCRIPT).map_err(|e| format!("Could not install visual script: {e}"))?;
    let license_path = scripts.join("XP-Enhance-RXP-LICENSE.txt");
    if !license_path.exists() { fs::write(&license_path, RXP_LICENSE).map_err(|e| format!("Could not write bundled license: {e}"))?; }
    let ini_path = scripts.join("rxp-xp12-enhancer.ini");
    if ini_path.exists() { backup_existing(&ini_path)?; }
    fs::write(&ini_path, settings_ini(&settings)).map_err(|e| format!("Could not write simulator settings: {e}"))?;
    let fps_path = scripts.join("xp-enhance-fps.lua");
    if fps_path.exists() { backup_existing(&fps_path)?; }
    fs::write(&fps_path, fps_overlay_source(&settings)).map_err(|e| format!("Could not install FPS overlay: {e}"))?;
    Ok("Visual bridge installed. Existing script and settings files were backed up where necessary. Reload FlyWithLua in X-Plane; upstream testing was reported for X-Plane 12.4.0, so newer versions require verification.".into())
}
#[tauri::command]
fn deploy_reshade_preset(path: String, preset: String) -> Result<String, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The X-Plane directory is not valid.".into()); }
    if preset.trim().is_empty() { return Err("No ReShade preset content was supplied.".into()); }
    if preset.len() > 2_000_000 { return Err("The ReShade preset exceeds the 2 MB limit.".into()); }
    if !preset.lines().any(|line| { let line = line.trim(); line.starts_with('[') && line.ends_with(']') }) { return Err("This file does not look like a ReShade INI preset.".into()); }
    let target = root.join("XPEnhance-ReShade.ini");
    if target.exists() { backup_existing(&target)?; }
    fs::write(&target, preset).map_err(|e| format!("Could not copy preset into X-Plane: {e}"))?;
    Ok("Preset copied to the X-Plane folder as XPEnhance-ReShade.ini. Select it in the ReShade overlay after installing the official Vulkan runtime and all referenced shader packages.".into())
}
#[tauri::command]
fn launch_lumina_manager(path: String) -> Result<String, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The X-Plane directory is not valid.".into()); }
    let manager = root.join("Resources/plugins/LuminaXP/LuminaXP.exe");
    if !manager.is_file() { return Err("LuminaXP.exe was not found. Download the official LuminaXP package and extract its LuminaXP folder into Resources/plugins first.".into()); }
    std::process::Command::new(&manager).current_dir(manager.parent().unwrap_or(&root)).spawn().map_err(|e| format!("Could not launch LuminaXP manager: {e}"))?;
    Ok("LuminaXP manager launched. Its settings and diagnostics are provided by the plugin developer.".into())
}

#[tauri::command]

#[tauri::command]
fn install_fps_overlay(path: String, settings: AppSettings) -> Result<String, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The X-Plane directory is not valid.".into()); }
    let fwl = flywithlua_dir(&root).ok_or("FlyWithLua NG was not found. Install FlyWithLua NG before installing the in-simulator FPS overlay.")?;
    let scripts = fwl.join("Scripts");
    fs::create_dir_all(&scripts).map_err(|e| format!("Could not create FlyWithLua Scripts folder: {e}"))?;
    let target = scripts.join("xp-enhance-fps.lua");
    if target.exists() { backup_existing(&target)?; }
    fs::write(&target, fps_overlay_source(&settings)).map_err(|e| format!("Could not install FPS overlay: {e}"))?;
    Ok("FPS overlay installed. Reload FlyWithLua scripts in X-Plane; use Settings to toggle it and choose its screen corner.".into())
}

fn json_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|v| if let Some(s) = v.as_str() { Some(s.to_string()) } else if v.is_number() { Some(v.to_string()) } else { None })
}

fn xplane_dataref_id(client: &reqwest::blocking::Client, port: u16, name: &str) -> Result<u64, String> {
    let response = client.get(format!("http://127.0.0.1:{port}/api/v1/datarefs"))
        .query(&[("filter[name]", name)])
        .header("Accept", "application/json")
        .send().map_err(|e| format!("Could not reach X-Plane's local Web API on port {port}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("X-Plane Web API returned HTTP {}. Start X-Plane 12.1.1+ and enable incoming traffic in Settings → Network.", response.status()));
    }
    let payload: Value = response.json().map_err(|e| format!("Could not parse X-Plane dataref response: {e}"))?;
    payload.get("data").and_then(Value::as_array).and_then(|rows| rows.iter().find(|row| row.get("name").and_then(Value::as_str) == Some(name)))
        .and_then(|row| row.get("id")).and_then(Value::as_u64)
        .ok_or_else(|| format!("X-Plane does not expose the required dataref '{name}'. Check your simulator version."))?
}
fn set_xplane_dataref(port: u16, name: &str, value: i64) -> Result<(), String> {
    if port == 0 { return Err("X-Plane Web API port must be between 1 and 65535.".into()); }
    let client = reqwest::blocking::Client::builder().user_agent("XP-Enhance/0.5.0").timeout(Duration::from_secs(8)).build().map_err(|e| e.to_string())?;
    let id = xplane_dataref_id(&client, port, name)?;
    let response = client.patch(format!("http://127.0.0.1:{port}/api/v1/datarefs/{id}/value"))
        .header("Accept", "application/json")
        .json(&serde_json::json!({"data": value}))
        .send().map_err(|e| format!("Could not write X-Plane dataref '{name}': {e}"))?;
    if !response.status().is_success() {
        return Err(format!("X-Plane rejected '{name}' with HTTP {}. The dataref may be read-only in this simulator build.", response.status()));
    }
    Ok(())
}
#[tauri::command]
fn apply_weather_preset(port: u16, preset: u8) -> Result<String, String> {
    if preset > 8 { return Err("Weather preset must be between 0 and 8.".into()); }
    set_xplane_dataref(port, "sim/weather/region/weather_preset", preset as i64)?;
    let _ = set_xplane_dataref(port, "sim/weather/region/update_immediately", 1);
    let labels = ["Clear", "VFR Few", "VFR Scattered", "VFR Broken", "VFR Marginal", "IFR Non-precision", "IFR Precision", "Convective", "Large-cell Storms"];
    Ok(format!("Applied the {} weather preset through X-Plane's local Web API. Cloud layers may take around a minute to settle.", labels[preset as usize]))
}
#[tauri::command]
fn set_live_weather(port: u16) -> Result<String, String> {
    set_xplane_dataref(port, "sim/weather/region/change_mode", 7)?;
    let _ = set_xplane_dataref(port, "sim/weather/region/update_immediately", 1);
    Ok("Requested X-Plane real-weather mode. The simulator's own weather source controls the resulting conditions.".into())
}

#[tauri::command]
fn fetch_metar(icao: String) -> Result<MetarReport, String> {
    let station = icao.trim().to_ascii_uppercase();
    if station.len() < 3 || station.len() > 4 || !station.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Enter a valid 3- or 4-character ICAO station code.".into());
    }
    let client = reqwest::blocking::Client::builder().user_agent("XP-Enhance/0.5.0").timeout(Duration::from_secs(20)).build().map_err(|e| e.to_string())?;
    let url = format!("https://aviationweather.gov/api/data/metar?ids={station}&format=json");
    let response = client.get(url).send().map_err(|e| format!("METAR request failed: {e}"))?;
    if !response.status().is_success() { return Err(format!("Weather service returned HTTP {}", response.status())); }
    let rows: Vec<Value> = response.json().map_err(|e| format!("Could not parse METAR response: {e}"))?;
    let row = rows.first().ok_or_else(|| format!("No current METAR was found for {station}."))?;
    let wind = match (json_field(row, "wdir"), json_field(row, "wspd")) {
        (Some(dir), Some(speed)) => Some(format!("{dir}° at {speed} kt")),
        (_, Some(speed)) => Some(format!("Variable at {speed} kt")),
        _ => None,
    };
    Ok(MetarReport { station, observed: json_field(row, "obsTime"), temperature_c: json_field(row, "temp"), dewpoint_c: json_field(row, "dewp"), wind,
        visibility_sm: json_field(row, "visib"), altimeter: json_field(row, "altim"), raw: json_field(row, "rawOb") })
}

#[tauri::command]
fn get_performance_snapshot() -> Result<PerformanceSnapshot, String> {
    let mut system = System::new_all();
    system.refresh_all();
    std::thread::sleep(Duration::from_millis(120));
    system.refresh_all();
    let cpu_usage_percent = system.global_cpu_usage();
    let memory_used_mb = system.used_memory() / (1024 * 1024);
    let memory_total_mb = system.total_memory() / (1024 * 1024);
    let mut gpu_name = None; let mut gpu_utilization_percent = None; let mut gpu_temperature_c = None; let mut vram_used_mb = None; let mut vram_total_mb = None;
    if let Ok(output) = Command::new("nvidia-smi").args(["--query-gpu=name,utilization.gpu,temperature.gpu,memory.used,memory.total", "--format=csv,noheader,nounits"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let parts: Vec<&str> = line.split(',').map(str::trim).collect();
                if parts.len() >= 5 {
                    gpu_name = Some(parts[0].to_string());
                    gpu_utilization_percent = parts[1].parse().ok(); gpu_temperature_c = parts[2].parse().ok();
                    vram_used_mb = parts[3].parse().ok(); vram_total_mb = parts[4].parse().ok();
                }
            }
        }
    }
    Ok(PerformanceSnapshot { cpu_usage_percent, memory_used_mb, memory_total_mb, gpu_name, gpu_utilization_percent, gpu_temperature_c, vram_used_mb, vram_total_mb })
}

#[tauri::command]
fn validate_reshade_preset(path: String, preset: String) -> Result<ReshadeReport, String> {
    let root = PathBuf::from(path.trim());
    if !valid_xplane(&root) { return Err("The X-Plane directory is not valid.".into()); }
    if preset.len() > 2_000_000 { return Err("The ReShade preset exceeds the 2 MB limit.".into()); }
    let sections = preset.lines().filter(|line| { let l=line.trim(); l.starts_with('[') && l.ends_with(']') }).count();
    let mut effect_search_paths=Vec::new(); let mut texture_search_paths=Vec::new(); let mut missing_paths=Vec::new();
    for line in preset.lines() {
        let Some((key,value)) = line.split_once('=') else { continue; };
        let key=key.trim().to_ascii_lowercase();
        if key != "effectsearchpaths" && key != "texturesearchpaths" { continue; }
        for item in value.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let clean=item.trim_matches('"');
            let target=PathBuf::from(clean);
            let resolved=if target.is_absolute() { target } else { root.join(target) };
            if key == "effectsearchpaths" { effect_search_paths.push(clean.to_string()); } else { texture_search_paths.push(clean.to_string()); }
            if !resolved.exists() { missing_paths.push(clean.to_string()); }
        }
    }
    Ok(ReshadeReport { sections, effect_search_paths, texture_search_paths, missing_paths })
}

fn open_external(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) { return Err("Only HTTP and HTTPS links can be opened.".into()); }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/C", "start", ""]).arg(&url).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&url).spawn().map_err(|e| e.to_string())?; }
    #[cfg(all(unix, not(target_os = "macos")))]
    { std::process::Command::new("xdg-open").arg(&url).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}

#[derive(Serialize)]
struct UpdateInfo { version: String, current_version: String }

fn latest_release_details() -> Result<(String, String, String), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("XP-Enhance-Updater")
        .timeout(Duration::from_secs(45))
        .build().map_err(|e| format!("Could not initialise update checker: {e}"))?;
    let response = client.get("https://api.github.com/repos/APGSwizzVR/XP-Enhance/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send().map_err(|e| format!("Could not contact GitHub releases: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("GitHub release check returned HTTP {}", response.status()));
    }
    let release: Value = response.json().map_err(|e| format!("Could not read release metadata: {e}"))?;
    let tag = release.get("tag_name").and_then(Value::as_str)
        .ok_or_else(|| "Latest GitHub release has no version tag.".to_string())?.to_string();
    let mut selected: Option<(String, String)> = None;
    if let Some(assets) = release.get("assets").and_then(Value::as_array) {
        for asset in assets {
            let name = asset.get("name").and_then(Value::as_str).unwrap_or("");
            let url = asset.get("browser_download_url").and_then(Value::as_str).unwrap_or("");
            let digest = asset.get("digest").and_then(Value::as_str).unwrap_or("");
            if name.to_ascii_lowercase().ends_with(".exe")
                && url.starts_with("https://github.com/APGSwizzVR/XP-Enhance/releases/download/")
                && digest.starts_with("sha256:") {
                selected = Some((url.to_string(), digest.to_string()));
                break;
            }
        }
    }
    let (url, digest) = selected.ok_or_else(|| "The latest release has no Windows installer with a SHA-256 digest. Please try again later.".to_string())?;
    Ok((tag, url, digest))
}

#[tauri::command]
fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let (tag, _, _) = latest_release_details()?;
    let latest_text = tag.trim_start_matches('v');
    let latest = semver::Version::parse(latest_text).map_err(|e| format!("Invalid release version '{tag}': {e}"))?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    if latest > current {
        Ok(Some(UpdateInfo { version: tag, current_version: current.to_string() }))
    } else { Ok(None) }
}

#[tauri::command]
fn install_update(app: AppHandle, version: String) -> Result<String, String> {
    let (latest_tag, url, expected_digest) = latest_release_details()?;
    if version != latest_tag {
        return Err("The release changed since the update notice appeared. Reopen XP Enhance to check again.".into());
    }
    let latest = semver::Version::parse(latest_tag.trim_start_matches('v')).map_err(|e| e.to_string())?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    if latest <= current { return Err("XP Enhance is already up to date.".into()); }

    let client = reqwest::blocking::Client::builder()
        .user_agent("XP-Enhance-Updater")
        .timeout(Duration::from_secs(180))
        .build().map_err(|e| e.to_string())?;
    let response = client.get(&url).send().map_err(|e| format!("Installer download failed: {e}"))?;
    if !response.status().is_success() { return Err(format!("Installer download returned HTTP {}", response.status())); }
    if response.content_length().unwrap_or(0) > 500 * 1024 * 1024 {
        return Err("The installer is larger than the 500 MB safety limit.".into());
    }
    let bytes = response.bytes().map_err(|e| format!("Could not read installer download: {e}"))?;
    if bytes.len() > 500 * 1024 * 1024 { return Err("The installer is larger than the 500 MB safety limit.".into()); }
    let expected = expected_digest.strip_prefix("sha256:").ok_or_else(|| "Release digest format is unsupported.".to_string())?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if !actual.eq_ignore_ascii_case(expected) { return Err("Installer integrity check failed. The downloaded file was not launched.".into()); }

    let safe_version: String = version.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' }).collect();
    let installer = std::env::temp_dir().join(format!("XP-Enhance-Setup-{safe_version}.exe"));
    fs::write(&installer, &bytes).map_err(|e| format!("Could not save installer: {e}"))?;
    #[cfg(target_os = "windows")]
    {
        Command::new(&installer).spawn().map_err(|e| format!("Could not launch verified installer: {e}"))?;
        app.exit(0);
        Ok("Verified installer launched. XP Enhance is closing so the update can be installed.".into())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        let _ = installer;
        Err("In-app installer is currently supported on Windows only.".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings, save_settings, detect_xplane, scan_xplane, get_integration_status, sync_visual_settings, install_visual_integration, deploy_reshade_preset, launch_lumina_manager, open_external, check_for_update, install_update, install_fps_overlay, fetch_metar, get_performance_snapshot, validate_reshade_preset, apply_weather_preset, set_live_weather])
        .run(tauri::generate_context!())
        .expect("error while running XP Enhance");
}
