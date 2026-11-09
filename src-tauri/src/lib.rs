use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};

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
}
impl Default for AppSettings {
    fn default() -> Self {
        Self { preset: "Balanced".into(), realism: 68, xplane_path: None, reshade_ini: None, features: FeatureSettings::default(), weather_profile: "Live weather".into(), scenery_profile: "Balanced".into(), telemetry_enabled: true }
    }
}

#[derive(Serialize)]
struct Addon { name: String, path: String, kind: String }
#[derive(Serialize)]
struct ScanResult { plugins: Vec<Addon>, scenery: Vec<Addon>, aircraft: Vec<Addon>, xplane_found: bool, version: Option<String> }
#[derive(Serialize)]
struct IntegrationStatus { flywithlua_installed: bool, visual_script_installed: bool, reshade_detected: bool, lumina_detected: bool, lumina_manager_found: bool, lumina_plugin_binary_found: bool }

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
    Ok(IntegrationStatus { flywithlua_installed: fwl.is_some(), visual_script_installed: script_installed, reshade_detected: reshade_found(&root), lumina_detected: lumina, lumina_manager_found, lumina_plugin_binary_found })
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
fn write_bridge_settings(root: &Path, settings: &AppSettings) -> Result<String, String> {
    if !valid_xplane(root) { return Err("The X-Plane directory is not valid.".into()); }
    let fwl = flywithlua_dir(root).ok_or("FlyWithLua NG was not found. Install FlyWithLua NG first, then try again.")?;
    let scripts = fwl.join("Scripts");
    fs::create_dir_all(&scripts).map_err(|e| format!("Could not create FlyWithLua Scripts folder: {e}"))?;
    let script = scripts.join("rxp-xp12-enhancer.lua");
    if !script.is_file() { return Err("The XP Enhance visual script is not installed yet. Use Install visual bridge first.".into()); }
    let ini = scripts.join("rxp-xp12-enhancer.ini");
    if ini.exists() { fs::write(&ini, settings_ini(settings)).map_err(|e| format!("Could not update simulator settings: {e}"))?; }
    else { fs::write(&ini, settings_ini(settings)).map_err(|e| format!("Could not write simulator settings: {e}"))?; }
    Ok("Simulator settings file updated. Reload FlyWithLua scripts in X-Plane to apply the changes.".into())
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings, save_settings, detect_xplane, scan_xplane, get_integration_status, sync_visual_settings, install_visual_integration, deploy_reshade_preset, launch_lumina_manager, open_external])
        .run(tauri::generate_context!())
        .expect("error while running XP Enhance");
}
