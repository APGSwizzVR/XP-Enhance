use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, process::Command, sync::{Mutex, OnceLock}, time::Duration};
use sysinfo::System;
use reqwest::blocking::Client;
use serde_json::Value;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct FeatureSettings {
    colour: bool, sharpening: bool, ambient_occlusion: bool, bloom: bool,
    cloud_enhancement: bool, anti_aliasing: bool, depth_of_field: bool,
    water_reflections: bool, night_lighting: bool, terrain_detail: bool,
}
impl Default for FeatureSettings {
    fn default() -> Self { Self { colour:true, sharpening:true, ambient_occlusion:true, bloom:false, cloud_enhancement:true, anti_aliasing:true, depth_of_field:false, water_reflections:true, night_lighting:true, terrain_detail:true } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct DlssSettings {
    gpu_series:String, style:String, strength:u8, render_scale:u8, sharpness:u8,
    lighting:u8, materials:u8, temporal:u8, motion:u8, ghosting:u8, exposure:u8,
    colour_depth:u8, foliage:u8, clouds:u8, cockpit:u8, reflections:u8,
    frame_generation:bool, ray_reconstruction:bool, low_latency:bool,
    dynamic_resolution:bool, preserve_grain:bool,
}
impl Default for DlssSettings {
    fn default() -> Self { Self { gpu_series:"Auto detect".into(), style:"Natural".into(), strength:65, render_scale:100, sharpness:35, lighting:70, materials:65, temporal:75, motion:70, ghosting:65, exposure:55, colour_depth:70, foliage:65, clouds:70, cockpit:85, reflections:65, frame_generation:false, ray_reconstruction:false, low_latency:true, dynamic_resolution:false, preserve_grain:true } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    preset:String, realism:u8, xplane_path:Option<String>, reshade_ini:Option<String>,
    features:FeatureSettings, weather_profile:String, scenery_profile:String,
    telemetry_enabled:bool, fps_overlay_enabled:bool, fps_overlay_position:String,
    xplane_api_port:u16, dlss:DlssSettings, atmosphere:HashMap<String,u8>,
}
impl Default for AppSettings {
    fn default() -> Self { Self { preset:"Balanced".into(), realism:68, xplane_path:None, reshade_ini:None, features:FeatureSettings::default(), weather_profile:"Clear morning".into(), scenery_profile:"Balanced".into(), telemetry_enabled:true, fps_overlay_enabled:false, fps_overlay_position:"top-right".into(), xplane_api_port:8086, dlss:DlssSettings::default(), atmosphere:HashMap::new() } }
}

#[derive(Serialize)]
struct Addon { name:String, path:String, kind:String }
#[derive(Serialize)]
struct ScanResult { plugins:Vec<Addon>, scenery:Vec<Addon>, aircraft:Vec<Addon>, xplane_found:bool, version:Option<String> }
#[derive(Serialize)]
struct PerformanceSnapshot {
    cpu_usage_percent:f32, memory_used_mb:u64, memory_total_mb:u64,
    gpu_name:Option<String>, gpu_utilization_percent:Option<u32>,
    gpu_temperature_c:Option<u32>, vram_used_mb:Option<u64>, vram_total_mb:Option<u64>,
    process_count:usize, xplane_running:bool, xplane_cpu_usage_percent:Option<f32>,
    xplane_memory_mb:Option<u64>, system_uptime_seconds:u64, xplane_fps:Option<f32>,
    xplane_frame_time_ms:Option<f32>, xplane_cpu_frame_ms:Option<f32>, xplane_gpu_frame_ms:Option<f32>,
    cpu_core_count:Option<usize>, logical_thread_count:usize, cpu_frequency_mhz:Option<u64>,
    memory_available_mb:u64, swap_used_mb:u64, swap_total_mb:u64,
}

fn settings_file(app:&AppHandle)->Result<PathBuf,String>{
    let dir=app.path().app_config_dir().map_err(|e|e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e|format!("Could not create settings directory: {e}"))?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
fn get_settings(app:AppHandle)->Result<AppSettings,String>{
    let path=settings_file(&app)?;
    if !path.exists(){return Ok(AppSettings::default())}
    let bytes=fs::read(path).map_err(|e|format!("Could not read settings: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e|format!("Settings file is invalid: {e}"))
}

#[tauri::command]
fn save_settings(app:AppHandle,settings:AppSettings)->Result<(),String>{
    let path=settings_file(&app)?;
    let bytes=serde_json::to_vec_pretty(&settings).map_err(|e|e.to_string())?;
    fs::write(&path,bytes).map_err(|e|format!("Could not write settings: {e}"))?;
    Ok(())
}

fn valid_xplane(path:&Path)->bool{path.join("X-Plane.exe").is_file()}

#[tauri::command]
fn detect_xplane()->Option<String>{
    let mut candidates:Vec<PathBuf>=Vec::new();
    for key in ["PROGRAMFILES","PROGRAMFILES(X86)","USERPROFILE"] {
        if let Some(base)=std::env::var_os(key){let b=PathBuf::from(base);candidates.push(b.join("X-Plane 12"));}
    }
    for drive in ["C:\\","D:\\","E:\\","F:\\","G:\\"] {candidates.push(PathBuf::from(drive).join("X-Plane 12"));}
    for candidate in candidates {if valid_xplane(&candidate){return Some(candidate.to_string_lossy().into_owned())}}
    None
}

fn directories(path:&Path,kind:&str)->Vec<Addon>{
    let mut items=Vec::new();
    let Ok(entries)=fs::read_dir(path) else{return items};
    for entry in entries.flatten(){
        let Ok(file_type)=entry.file_type() else{continue};
        if !file_type.is_dir(){continue}
        let full=entry.path();
        let name=entry.file_name().to_string_lossy().into_owned();
        items.push(Addon{name,path:full.to_string_lossy().into_owned(),kind:kind.into()});
    }
    items.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}

#[tauri::command]
fn scan_xplane(path:String)->Result<ScanResult,String>{
    let root=PathBuf::from(path.trim());
    if !valid_xplane(&root){return Err("The selected folder does not contain X-Plane.exe.".into())}
    let resources=root.join("Resources");
    let plugins=directories(&resources.join("plugins"),"Plugin");
    let scenery=directories(&root.join("Custom Scenery"),"Scenery");
    let aircraft=directories(&root.join("Aircraft"),"Aircraft");
    let version_path=resources.join("version.txt");
    let version=fs::read_to_string(version_path).ok().map(|s|s.trim().to_string()).filter(|s|!s.is_empty());
    Ok(ScanResult{plugins,scenery,aircraft,xplane_found:true,version})
}

static TELEMETRY_SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();
static TELEMETRY_HTTP: OnceLock<Client> = OnceLock::new();
static DATAREF_IDS: OnceLock<Mutex<HashMap<(u16, String), Option<u64>>>> = OnceLock::new();

fn telemetry_client() -> &'static Client {
    TELEMETRY_HTTP.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_millis(150))
            .timeout(Duration::from_millis(250))
            .pool_max_idle_per_host(4)
            .build()
            .expect("telemetry HTTP client configuration is valid")
    })
}

fn xplane_dataref(client: &Client, port: u16, name: &str) -> Option<f32> {
    let cache = DATAREF_IDS.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (port, name.to_string());
    let cached = cache.lock().ok()?.get(&key).copied();
    let id = match cached {
        Some(Some(id)) => id,
        Some(None) => return None,
        None => {
            let base = format!("http://127.0.0.1:{port}/api/v1");
            let list: Value = client
                .get(format!("{base}/datarefs"))
                .query(&[("filter[name]", name)])
                .send().ok()?.error_for_status().ok()?.json().ok()?;
            let found = list.get("data")?.as_array()?.first()
                .and_then(|v| v.get("id"))
                .and_then(Value::as_u64);
            if let Ok(mut ids) = cache.lock() { ids.insert(key, found); }
            found?
        }
    };
    let value: Value = client
        .get(format!("http://127.0.0.1:{port}/api/v1/datarefs/{id}/value"))
        .send().ok()?.error_for_status().ok()?.json().ok()?;
    value.get("data")?.as_f64().map(|v| v as f32)
}

#[tauri::command]
fn get_performance_snapshot(xplane_api_port: u16) -> Result<PerformanceSnapshot, String> {
    // Keep one sysinfo instance between samples. Refresh only the data we display;
    // do not enumerate every system subsystem or sleep on the UI command thread.
    let system = TELEMETRY_SYSTEM.get_or_init(|| Mutex::new(System::new()));
    let (cpu_usage_percent, memory_used_mb, memory_total_mb, memory_available_mb,
        swap_used_mb, swap_total_mb, cpu_core_count, logical_thread_count,
        cpu_frequency_mhz, process_count, xplane_running, xplane_cpu_usage_percent,
        xplane_memory_mb) = {
        let mut system = system.lock().map_err(|_| "Telemetry system lock was poisoned".to_string())?;
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let cpu_usage_percent = system.global_cpu_usage();
        let memory_used_mb = system.used_memory() / (1024 * 1024);
        let memory_total_mb = system.total_memory() / (1024 * 1024);
        let memory_available_mb = system.available_memory() / (1024 * 1024);
        let swap_used_mb = system.used_swap() / (1024 * 1024);
        let swap_total_mb = system.total_swap() / (1024 * 1024);
        let cpu_core_count = system.physical_core_count();
        let logical_thread_count = system.cpus().len();
        let cpu_frequency_mhz = system.cpus().first().map(|c| c.frequency());
        let process_count = system.processes().len();
        let xplane = system.processes().values().find(|p| {
            let n = p.name().to_string_lossy().to_ascii_lowercase();
            n == "x-plane.exe" || n == "x-plane"
        });
        let xplane_running = xplane.is_some();
        let xplane_cpu_usage_percent = xplane.map(|p| p.cpu_usage());
        let xplane_memory_mb = xplane.map(|p| p.memory() / (1024 * 1024));
        (cpu_usage_percent, memory_used_mb, memory_total_mb, memory_available_mb,
            swap_used_mb, swap_total_mb, cpu_core_count, logical_thread_count,
            cpu_frequency_mhz, process_count, xplane_running, xplane_cpu_usage_percent,
            xplane_memory_mb)
    };

    let system_uptime_seconds = System::uptime();
    // Never spawn PowerShell/WMI from telemetry. GPU telemetry is reported as
    // unavailable until it can be collected through a native, non-blocking API.
    let gpu_name = None;
    let gpu_utilization_percent = None;
    let vram_used_mb = None;
    let vram_total_mb = None;

    let (xplane_fps, xplane_frame_time_ms, xplane_cpu_frame_ms, xplane_gpu_frame_ms) =
        if xplane_running {
            let client = telemetry_client();
            let period = xplane_dataref(client, xplane_api_port, "sim/time/framerate_period")
                .or_else(|| xplane_dataref(client, xplane_api_port, "sim/operation/misc/frame_rate_period"));
            let fps = period.filter(|v| *v > 0.0).map(|v| 1.0 / v);
            let frame_ms = period.map(|v| v * 1000.0);
            let cpu_ms = xplane_dataref(client, xplane_api_port, "sim/time/cpu_time_per_frame_sec_approx").map(|v| v * 1000.0);
            let gpu_ms = xplane_dataref(client, xplane_api_port, "sim/time/gpu_time_per_frame_sec_approx").map(|v| v * 1000.0);
            (fps, frame_ms, cpu_ms, gpu_ms)
        } else {
            (None, None, None, None)
        };

    Ok(PerformanceSnapshot {
        cpu_usage_percent, memory_used_mb, memory_total_mb, gpu_name,
        gpu_utilization_percent, gpu_temperature_c: None, vram_used_mb, vram_total_mb,
        process_count, xplane_running, xplane_cpu_usage_percent, xplane_memory_mb,
        system_uptime_seconds, xplane_fps, xplane_frame_time_ms, xplane_cpu_frame_ms,
        xplane_gpu_frame_ms, cpu_core_count, logical_thread_count, cpu_frequency_mhz,
        memory_available_mb, swap_used_mb, swap_total_mb,
    })
}

#[tauri::command]
fn set_xplane_fps_overlay(xplane_api_port:u16)->Result<(),String>{
    let client=Client::builder().timeout(Duration::from_secs(2)).build().map_err(|e|e.to_string())?;
    let base=format!("http://127.0.0.1:{xplane_api_port}/api/v2");
    let commands:Value=client.get(format!("{base}/commands")).query(&[("filter[name]","sim/operation/show_fps")]).send().map_err(|e|format!("Cannot reach X-Plane Web API: {e}"))?.error_for_status().map_err(|e|format!("X-Plane API request failed: {e}"))?.json().map_err(|e|format!("Invalid X-Plane API response: {e}"))?;
    let id=commands.get("data").and_then(Value::as_array).and_then(|a|a.first()).and_then(|v|v.get("id")).and_then(Value::as_u64).ok_or_else(||"X-Plane did not expose its built-in FPS display command. Check that a flight is loaded.".to_string())?;
    client.post(format!("{base}/command/{id}/activate")).json(&serde_json::json!({"duration":0})).send().map_err(|e|format!("Could not toggle X-Plane FPS display: {e}"))?.error_for_status().map_err(|e|format!("X-Plane rejected the FPS display command: {e}"))?;
    Ok(())
}


#[derive(Serialize)]
struct ReshadeStatus { detected:bool, config_found:bool, shader_folder_found:bool, global_vulkan_layer_found:bool, preset_folder:String }

#[tauri::command]
fn get_reshade_status(xplane_path:String)->Result<ReshadeStatus,String>{
    let root=PathBuf::from(xplane_path.trim());
    if !valid_xplane(&root){return Err("Select a valid X-Plane 12 folder first.".into())}
    let config_found=root.join("ReShade.ini").is_file() || root.join("reshade.ini").is_file();
    let shader_folder_found=root.join("reshade-shaders").is_dir();
    let global=std::env::var_os("PROGRAMDATA").map(PathBuf::from).unwrap_or_else(||PathBuf::from("C:/ProgramData")).join("ReShade");
    let global_vulkan_layer_found=global.exists() && fs::read_dir(&global).map(|it|it.flatten().any(|e|e.path().extension().and_then(|x|x.to_str()).map(|x|x.eq_ignore_ascii_case("json")).unwrap_or(false))).unwrap_or(false);
    let preset_folder=root.join("XPEnhance-Presets");
    Ok(ReshadeStatus{detected:config_found||shader_folder_found||global_vulkan_layer_found,config_found,shader_folder_found,global_vulkan_layer_found,preset_folder:preset_folder.to_string_lossy().into_owned()})
}

#[tauri::command]
fn launch_reshade_setup(xplane_path:String)->Result<String,String>{
    let root=PathBuf::from(xplane_path.trim());
    if !valid_xplane(&root){return Err("Select and save your X-Plane 12 folder in Settings first.".into())}
    let downloads=std::env::var_os("USERPROFILE").map(PathBuf::from).map(|p|p.join("Downloads"));
    let mut installer:Option<PathBuf>=None;
    if let Some(dir)=downloads {
        if let Ok(entries)=fs::read_dir(dir){
            let mut candidates:Vec<PathBuf>=entries.flatten().map(|e|e.path()).filter(|p|p.extension().and_then(|x|x.to_str()).map(|x|x.eq_ignore_ascii_case("exe")).unwrap_or(false) && p.file_name().and_then(|x|x.to_str()).map(|x|x.to_ascii_lowercase().contains("reshade") && x.to_ascii_lowercase().contains("setup")).unwrap_or(false)).collect();
            candidates.sort_by_key(|p|fs::metadata(p).and_then(|m|m.modified()).ok());
            installer=candidates.pop();
        }
    }
    if let Some(exe)=installer {
        Command::new(exe).spawn().map_err(|e|format!("Could not start ReShade setup: {e}"))?;
        Ok("Started the ReShade setup found in Downloads. Select X-Plane.exe and choose Vulkan.".into())
    } else {
        Command::new("cmd").args(["/C","start","","https://reshade.me/"]).spawn().map_err(|e|format!("Could not open the official ReShade download page: {e}"))?;
        Ok("Opened the official ReShade download page. Download and run its setup, select X-Plane.exe, then choose Vulkan.".into())
    }
}

#[tauri::command]
fn install_xp_enhance_effects(xplane_path:String)->Result<String,String>{
    let root=PathBuf::from(xplane_path.trim());
    if !valid_xplane(&root){return Err("Select a valid X-Plane 12 folder first.".into())}
    let target_dir=root.join("reshade-shaders").join("Shaders").join("XPEnhance");
    fs::create_dir_all(&target_dir).map_err(|e|format!("Could not create XP Enhance shader folder: {e}"))?;
    let target=target_dir.join("XPEnhance.fx");
    if target.exists(){
        let backup_dir=root.join("XPEnhance-Backups");
        fs::create_dir_all(&backup_dir).map_err(|e|e.to_string())?;
        let backup=backup_dir.join("XPEnhance.fx.previous");
        if !backup.exists(){fs::copy(&target,&backup).map_err(|e|format!("Could not back up the existing XP Enhance shader: {e}"))?;}
    }
    fs::write(&target,include_str!("../assets/XPEnhance.fx")).map_err(|e|format!("Could not install XP Enhance shader: {e}"))?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
fn deploy_reshade_preset(source_path:String,xplane_path:String)->Result<String,String>{
    let source=PathBuf::from(source_path.trim());
    let root=PathBuf::from(xplane_path.trim());
    if !valid_xplane(&root){return Err("Select a valid X-Plane 12 folder first.".into())}
    if !source.is_file() || source.extension().and_then(|x|x.to_str()).map(|x|!x.eq_ignore_ascii_case("ini")).unwrap_or(true){return Err("Choose an existing ReShade .ini preset file.".into())}
    let dir=root.join("XPEnhance-Presets");
    fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let name=source.file_name().ok_or_else(||"Preset filename is invalid.".to_string())?;
    let target=dir.join(name);
    if target.exists(){let backup=dir.join(format!("{}.backup",name.to_string_lossy()));if !backup.exists(){fs::copy(&target,&backup).map_err(|e|format!("Could not back up the existing preset: {e}"))?;}}
    fs::copy(&source,&target).map_err(|e|format!("Could not copy preset: {e}"))?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
fn open_xplane_folder(xplane_path:String)->Result<(),String>{
    let root=PathBuf::from(xplane_path.trim());
    if !valid_xplane(&root){return Err("Select a valid X-Plane 12 folder first.".into())}
    Command::new("explorer.exe").arg(root).spawn().map_err(|e|format!("Could not open X-Plane folder: {e}"))?;
    Ok(())
}

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings,save_settings,detect_xplane,scan_xplane,get_performance_snapshot,set_xplane_fps_overlay,get_reshade_status,launch_reshade_setup,install_xp_enhance_effects,deploy_reshade_preset,open_xplane_folder])
        .run(tauri::generate_context!())
        .expect("error while running XP Enhance");
}
