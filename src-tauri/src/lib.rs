use serde::{Deserialize,Serialize};
use std::{fs,path::{Path,PathBuf}};
use tauri::{AppHandle,Manager};
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(default)]
struct FeatureSettings{colour:bool,sharpening:bool,ambient_occlusion:bool,bloom:bool,cloud_enhancement:bool,anti_aliasing:bool,depth_of_field:bool,water_reflections:bool,night_lighting:bool,terrain_detail:bool}
impl Default for FeatureSettings{fn default()->Self{Self{colour:true,sharpening:true,ambient_occlusion:false,bloom:false,cloud_enhancement:true,anti_aliasing:true,depth_of_field:false,water_reflections:true,night_lighting:true,terrain_detail:true}}}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(default)]
struct AppSettings{preset:String,realism:u8,xplane_path:Option<String>,reshade_ini:Option<String>,features:FeatureSettings,weather_profile:String,scenery_profile:String,telemetry_enabled:bool}
impl Default for AppSettings{fn default()->Self{Self{preset:"Balanced".into(),realism:68,xplane_path:None,reshade_ini:None,features:FeatureSettings::default(),weather_profile:"Live weather".into(),scenery_profile:"Balanced".into(),telemetry_enabled:true}}}
#[derive(Serialize)]struct Addon{name:String,path:String,kind:String}
#[derive(Serialize)]struct ScanResult{plugins:Vec<Addon>,scenery:Vec<Addon>,aircraft:Vec<Addon>,xplane_found:bool,version:Option<String>}
fn settings_file(app:&AppHandle)->Result<PathBuf,String>{let d=app.path().app_config_dir().map_err(|e|e.to_string())?;fs::create_dir_all(&d).map_err(|e|e.to_string())?;Ok(d.join("settings.json"))}
#[tauri::command]fn get_settings(app:AppHandle)->Result<AppSettings,String>{let p=settings_file(&app)?;if !p.exists(){return Ok(AppSettings::default())}let c=fs::read_to_string(p).map_err(|e|e.to_string())?;serde_json::from_str(&c).map_err(|e|e.to_string())}
#[tauri::command]fn save_settings(app:AppHandle,settings:AppSettings)->Result<(),String>{let p=settings_file(&app)?;let c=serde_json::to_string_pretty(&settings).map_err(|e|e.to_string())?;let t=p.with_extension("json.tmp");fs::write(&t,c).map_err(|e|e.to_string())?;fs::rename(&t,&p).map_err(|e|e.to_string())}
fn valid(p:&Path)->bool{p.is_dir()&&(p.join("X-Plane.exe").is_file()||p.join("X-Plane 12.exe").is_file()||p.join("X-Plane-x86_64.exe").is_file())}
#[tauri::command]fn detect_xplane()->Option<String>{let mut c=Vec::new();if let Some(h)=std::env::var_os("USERPROFILE"){let h=PathBuf::from(h);c.extend([h.join("Desktop/X-Plane 12"),h.join("Documents/X-Plane 12"),h.join("X-Plane 12")]);}c.extend([PathBuf::from("C:/X-Plane 12"),PathBuf::from("D:/X-Plane 12"),PathBuf::from("C:/Program Files/X-Plane 12")]);c.into_iter().find(|p|valid(p)).map(|p|p.to_string_lossy().to_string())}
fn dirs(p:&Path,k:&str)->Vec<Addon>{let mut out=Vec::new();if let Ok(rd)=fs::read_dir(p){for e in rd.flatten(){let path=e.path();if path.is_dir(){out.push(Addon{name:e.file_name().to_string_lossy().to_string(),path:path.to_string_lossy().to_string(),kind:k.into()});}}}out.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()));out}
#[tauri::command]fn scan_xplane(path:String)->Result<ScanResult,String>{let root=PathBuf::from(path.trim());if !root.exists()||!root.is_dir(){return Err("The configured X-Plane directory does not exist.".into())}if !valid(&root){return Ok(ScanResult{plugins:vec![],scenery:vec![],aircraft:vec![],xplane_found:false,version:None})}Ok(ScanResult{plugins:dirs(&root.join("Resources/plugins"),"Plugin"),scenery:dirs(&root.join("Custom Scenery"),"Scenery"),aircraft:dirs(&root.join("Aircraft"),"Aircraft"),xplane_found:true,version:Some("X-Plane 12".into())})}
#[cfg_attr(mobile,tauri::mobile_entry_point)]pub fn run(){tauri::Builder::default().invoke_handler(tauri::generate_handler![get_settings,save_settings,detect_xplane,scan_xplane]).run(tauri::generate_context!()).expect("error while running XP Enhance");}