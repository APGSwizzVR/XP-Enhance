use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, process::Command, time::Duration};
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

fn xplane_dataref(client:&Client,port:u16,name:&str)->Option<f32>{
    let base=format!("http://127.0.0.1:{port}/api/v1");
    let list:Value=client.get(format!("{base}/datarefs")).query(&[("filter[name]",name)]).send().ok()?.error_for_status().ok()?.json().ok()?;
    let id=list.get("data")?.as_array()?.first()?.get("id")?.as_u64()?;
    let value:Value=client.get(format!("{base}/datarefs/{id}/value")).send().ok()?.error_for_status().ok()?.json().ok()?;
    value.get("data")?.as_f64().map(|v|v as f32)
}

#[tauri::command]
fn get_performance_snapshot(xplane_api_port:u16)->Result<PerformanceSnapshot,String>{
    let mut system=System::new_all(); system.refresh_all();
    std::thread::sleep(Duration::from_millis(120)); system.refresh_all();
    let cpu_usage_percent=system.global_cpu_usage();
    let memory_used_mb=system.used_memory()/(1024*1024);
    let memory_total_mb=system.total_memory()/(1024*1024);
    let memory_available_mb=system.available_memory()/(1024*1024);
    let swap_used_mb=system.used_swap()/(1024*1024);
    let swap_total_mb=system.total_swap()/(1024*1024);
    let cpu_core_count=System::physical_core_count();
    let logical_thread_count=system.cpus().len();
    let cpu_frequency_mhz=system.cpus().first().map(|c|c.frequency());
    let process_count=system.processes().len();
    let xplane=system.processes().values().find(|p|{let n=p.name().to_string_lossy().to_ascii_lowercase();n=="x-plane.exe"||n=="x-plane"});
    let xplane_running=xplane.is_some();
    let xplane_cpu_usage_percent=xplane.map(|p|p.cpu_usage());
    let xplane_memory_mb=xplane.map(|p|p.memory()/(1024*1024));
    let system_uptime_seconds=System::uptime();
    let gpu_name=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","(Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty Name)"]).output().ok().filter(|o|o.status.success()).and_then(|o|String::from_utf8_lossy(&o.stdout).lines().next().map(str::trim).filter(|s|!s.is_empty()).map(str::to_string));
    let mut gpu_utilization_percent=None;let mut vram_used_mb=None;let mut vram_total_mb=None;
    let client=Client::builder().timeout(Duration::from_millis(350)).build().map_err(|e|e.to_string())?;
    let period=xplane_dataref(&client,xplane_api_port,"sim/time/framerate_period").or_else(||xplane_dataref(&client,xplane_api_port,"sim/operation/misc/frame_rate_period"));
    let xplane_fps=period.filter(|v|*v>0.0).map(|v|1.0/v);
    let xplane_frame_time_ms=period.map(|v|v*1000.0);
    let xplane_cpu_frame_ms=xplane_dataref(&client,xplane_api_port,"sim/time/cpu_time_per_frame_sec_approx").map(|v|v*1000.0);
    let xplane_gpu_frame_ms=xplane_dataref(&client,xplane_api_port,"sim/time/gpu_time_per_frame_sec_approx").map(|v|v*1000.0);
    Ok(PerformanceSnapshot{cpu_usage_percent,memory_used_mb,memory_total_mb,gpu_name,gpu_utilization_percent,gpu_temperature_c:None,vram_used_mb,vram_total_mb,process_count,xplane_running,xplane_cpu_usage_percent,xplane_memory_mb,system_uptime_seconds,xplane_fps,xplane_frame_time_ms,xplane_cpu_frame_ms,xplane_gpu_frame_ms,cpu_core_count,logical_thread_count,cpu_frequency_mhz,memory_available_mb,swap_used_mb,swap_total_mb})
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

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings,save_settings,detect_xplane,scan_xplane,get_performance_snapshot,set_xplane_fps_overlay])
        .run(tauri::generate_context!())
        .expect("error while running XP Enhance");
}
