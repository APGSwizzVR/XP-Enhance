#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

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
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self {
            colour: true,
            sharpening: true,
            ambient_occlusion: false,
            bloom: false,
            cloud_enhancement: true,
            anti_aliasing: true,
            depth_of_field: false,
        }
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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            preset: "Balanced".to_string(),
            realism: 65,
            xplane_path: None,
            reshade_ini: None,
            features: FeatureSettings::default(),
        }
    }
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    Ok(directory.join("settings.json"))
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_file(&app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&contents).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_file(&app)?;
    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, contents).map_err(|e| e.to_string())?;
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_settings, save_settings])
        .run(tauri::generate_context!())
        .expect("error while running XP Enhance");
}

fn main() {
    xp_enhance_lib::run();
}