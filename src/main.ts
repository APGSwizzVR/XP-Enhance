import { invoke } from "@tauri-apps/api/core";
import "./style.css";

type Features = {
  colour: boolean;
  sharpening: boolean;
  ambient_occlusion: boolean;
  bloom: boolean;
  cloud_enhancement: boolean;
  anti_aliasing: boolean;
  depth_of_field: boolean;
};

type Settings = {
  preset: string;
  realism: number;
  xplane_path: string | null;
  reshade_ini: string | null;
  features: Features;
};

const defaults: Settings = {
  preset: "Balanced",
  realism: 65,
  xplane_path: null,
  reshade_ini: null,
  features: {
    colour: true,
    sharpening: true,
    ambient_occlusion: false,
    bloom: false,
    cloud_enhancement: true,
    anti_aliasing: true,
    depth_of_field: false
  }
};

let settings: Settings = structuredClone(defaults);
let activePage = "Dashboard";
let saveTimer: number | undefined;

const pages = ["Dashboard", "Visual Settings", "Presets", "ReShade", "Weather", "Scenery", "Community Addons", "Installed Addons", "Performance", "Settings"];
const icons: Record<string, string> = {
  Dashboard: "▦", "Visual Settings": "◈", Presets: "◫", ReShade: "◉",
  Weather: "☁", Scenery: "⌁", "Community Addons": "＋",
  "Installed Addons": "▤", Performance: "⌁", Settings: "⚙"
};

const app = document.querySelector<HTMLDivElement>("#app")!;

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, char => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[char] ?? char));
}

function queueSave(): void {
  window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(async () => {
    try {
      await invoke("save_settings", { settings });
      setStatus("All changes saved");
    } catch (error) {
      setStatus("Could not save settings: " + String(error));
    }
  }, 250);
}

function setStatus(message: string): void {
  const status = document.querySelector<HTMLElement>("#save-status");
  if (status) status.textContent = message;
}

function featureRow(key: keyof Features, title: string, description: string): string {
  const enabled = settings.features[key];
  return `<div class="feature-row">
    <div class="feature-copy"><strong>${title}</strong><span>${description}</span></div>
    <button class="switch ${enabled ? "on" : ""}" data-feature="${key}" role="switch" aria-checked="${enabled}" aria-label="Toggle ${title}"><span></span></button>
  </div>`;
}

function dashboard(): string {
  const enabledCount = Object.values(settings.features).filter(Boolean).length;
  return `<section class="welcome">
    <div><div class="eyebrow">FLIGHT EXPERIENCE CONTROL</div><h1>Good to have you here.</h1><p>Configure your X-Plane 12 visual experience from one place.</p></div>
    <div class="status-pill"><i></i> App ready</div>
  </section>
  <section class="stat-grid">
    <article class="stat-card"><span>ACTIVE PRESET</span><strong>${escapeHtml(settings.preset)}</strong><small>Current visual profile</small></article>
    <article class="stat-card"><span>REALISM LEVEL</span><strong>${settings.realism}<em>/100</em></strong><small>Overall enhancement intensity</small></article>
    <article class="stat-card"><span>FEATURES ENABLED</span><strong>${enabledCount}<em>/7</em></strong><small>Configured enhancement options</small></article>
  </section>
  <section class="content-grid">
    <article class="panel"><div class="panel-heading"><div><div class="eyebrow">QUICK CONTROL</div><h2>Realism intensity</h2></div><span class="value-chip">${settings.realism}%</span></div>
      <p class="muted">Adjust the overall strength of your visual profile.</p>
      <input class="range" id="realism-range" type="range" min="0" max="100" value="${settings.realism}" />
      <div class="range-labels"><span>Subtle</span><span>Balanced</span><span>Maximum</span></div>
      <div class="panel-footer"><span>Changes are stored locally</span><button class="text-button" data-page="Visual Settings">Open visual settings →</button></div>
    </article>
    <article class="panel"><div class="panel-heading"><div><div class="eyebrow">YOUR SETUP</div><h2>X-Plane connection</h2></div><span class="status-dot"></span></div>
      <div class="connection-line"><span class="connection-icon">XP</span><div><strong>${settings.xplane_path ? "Folder configured" : "Not configured"}</strong><small>${settings.xplane_path ? escapeHtml(settings.xplane_path) : "Set your simulator folder in Settings"}</small></div></div>
      <div class="panel-footer"><span>Simulator integration status</span><button class="text-button" data-page="Settings">Configure →</button></div>
    </article>
  </section>
  <section class="panel feature-panel"><div class="panel-heading"><div><div class="eyebrow">ACTIVE CONFIGURATION</div><h2>Visual features</h2></div><button class="text-button" data-page="Visual Settings">Manage features →</button></div>
    <div class="feature-grid">
      ${featureRow("colour", "Colour enhancement", "Colour and tone adjustments")}
      ${featureRow("sharpening", "Image sharpening", "Improve perceived image clarity")}
      ${featureRow("ambient_occlusion", "Ambient occlusion", "Additional contact shading")}
      ${featureRow("bloom", "Bloom", "Bright-light glow effect")}
      ${featureRow("cloud_enhancement", "Cloud enhancement", "Cloud visual profile")}
      ${featureRow("anti_aliasing", "Anti-aliasing", "Reduce visible edge aliasing")}
    </div>
  </section>`;
}

function visualSettings(): string {
  return `<div class="page-intro"><div class="eyebrow">CONFIGURATION</div><h1>Visual Settings</h1><p>Fine-tune your visual profile and feature preferences.</p></div>
  <section class="panel"><div class="panel-heading"><div><h2>Realism intensity</h2><p class="muted">A profile strength setting for your enhancement configuration.</p></div><span class="value-chip">${settings.realism}%</span></div>
    <input class="range" id="realism-range" type="range" min="0" max="100" value="${settings.realism}" /><div class="range-labels"><span>Subtle</span><span>Balanced</span><span>Maximum</span></div>
  </section>
  <section class="panel feature-panel"><div class="panel-heading"><div><h2>Enhancement features</h2><p class="muted">Enable or disable individual options.</p></div></div>
    ${featureRow("colour", "Colour enhancement", "Colour grading and tone adjustments")}
    ${featureRow("sharpening", "Image sharpening", "Improve perceived image clarity")}
    ${featureRow("ambient_occlusion", "Ambient occlusion", "Additional contact shading")}
    ${featureRow("bloom", "Bloom", "Bright-light glow effect")}
    ${featureRow("cloud_enhancement", "Cloud enhancement", "Cloud visual profile options")}
    ${featureRow("anti_aliasing", "Anti-aliasing", "Reduce visible edge aliasing")}
    ${featureRow("depth_of_field", "Depth of field", "Camera focus effect")}
  </section>
  <p class="notice">These settings currently save as an XP Enhance profile. Applying effects to X-Plane's live renderer is a separate integration task.</p>`;
}

function presets(): string {
  const options = [
    ["Performance", "Prioritise frame rate and lower visual overhead.", "Low"],
    ["Balanced", "A general-purpose profile for everyday flying.", "Medium"],
    ["Realistic", "A stronger visual profile for immersive flights.", "High"],
    ["Cinematic", "A more pronounced profile for screenshots and video.", "Very high"]
  ];
  return `<div class="page-intro"><div class="eyebrow">PROFILES</div><h1>Presets</h1><p>Choose a starting point for your visual configuration.</p></div>
  <div class="preset-grid">${options.map(([name,desc,load]) => `<button class="preset-card ${settings.preset === name ? "selected" : ""}" data-preset="${name}">
    <span class="preset-mark">${settings.preset === name ? "✓" : "◈"}</span><strong>${name}</strong><p>${desc}</p><small>INTENSITY · ${load}</small><span class="preset-action">${settings.preset === name ? "CURRENT PRESET" : "SELECT PRESET →"}</span>
  </button>`).join("")}</div>`;
}

function reshade(): string {
  return `<div class="page-intro"><div class="eyebrow">COMPATIBILITY</div><h1>ReShade</h1><p>Import and keep a ReShade configuration alongside your XP Enhance profile.</p></div>
  <section class="panel"><div class="eyebrow">INI CONFIGURATION</div><h2>Import ReShade preset</h2><p class="muted">Select a ReShade .ini file. XP Enhance will store a copy of its contents in your local settings.</p>
    <label class="file-drop"><span class="file-icon">＋</span><strong>Choose a ReShade .ini file</strong><small>Browse your computer · .ini files only</small><input type="file" id="reshade-file" accept=".ini,text/plain" /></label>
    <div class="import-state">${settings.reshade_ini ? '<span class="status-pill"><i></i> INI imported and saved</span>' : '<span class="muted">No INI file imported</span>'}</div>
  </section>
  <p class="notice">Import and storage are implemented. Loading ReShade shaders into X-Plane and translating individual shader parameters are not implemented yet.</p>`;
}

function settingsPage(): string {
  return `<div class="page-intro"><div class="eyebrow">APPLICATION</div><h1>Settings</h1><p>Configure XP Enhance and your simulator location.</p></div>
  <section class="panel"><h2>X-Plane 12 installation</h2><p class="muted">Enter the folder containing your X-Plane.exe file.</p>
    <label class="field-label" for="xplane-path">Simulator folder path</label><input class="text-input" id="xplane-path" placeholder="e.g. D:\\X-Plane 12" value="${escapeHtml(settings.xplane_path ?? "")}" />
    <div class="panel-footer"><span>Saved locally on this PC</span><button class="primary-button" id="save-path">Save folder</button></div>
  </section>
  <section class="panel"><h2>Application information</h2><div class="info-row"><span>Application</span><strong>XP Enhance</strong></div><div class="info-row"><span>Version</span><strong>0.2.0 · Rust / Tauri 2</strong></div><div class="info-row"><span>Settings storage</span><strong>Local JSON configuration</strong></div></section>`;
}

function simplePage(name: string): string {
  const descriptions: Record<string, string> = {
    Weather: "Weather profile controls and simulator weather integration.",
    Scenery: "Manage scenery locations and visual environment options.",
    "Community Addons": "Discover and install community enhancements.",
    "Installed Addons": "View and manage your installed simulator add-ons.",
    Performance: "Monitor and tune simulator performance settings."
  };
  return `<div class="page-intro"><div class="eyebrow">XP ENHANCE</div><h1>${name}</h1><p>${descriptions[name] ?? "Manage your XP Enhance configuration."}</p></div>
  <section class="panel empty-state"><div class="empty-symbol">◈</div><h2>This section is being built</h2><p class="muted">The Rust desktop foundation is in place. This module will be connected to X-Plane in a later development step.</p></section>`;
}

function render(): void {
  const body = activePage === "Dashboard" ? dashboard()
    : activePage === "Visual Settings" ? visualSettings()
    : activePage === "Presets" ? presets()
    : activePage === "ReShade" ? reshade()
    : activePage === "Settings" ? settingsPage()
    : simplePage(activePage);

  app.innerHTML = `<div class="shell">
    <aside class="sidebar">
      <div class="brand"><div class="brand-icon">X</div><div><strong>XP ENHANCE</strong><small>FLIGHT VISUAL CONTROL</small></div></div>
      <div class="nav-label">WORKSPACE</div>
      <nav>${pages.map(page => `<button class="nav-item ${activePage === page ? "active" : ""}" data-page="${page}"><span class="nav-icon">${icons[page]}</span>${page}</button>`).join("")}</nav>
      <div class="sidebar-bottom"><div class="sidebar-version">VERSION 0.2.0</div><div class="sidebar-status"><i></i> Desktop app ready</div></div>
    </aside>
    <main class="main-area">
      <header class="topbar"><div><span class="crumb">XP ENHANCE</span><span class="crumb-separator">/</span><strong>${activePage.toUpperCase()}</strong></div><div class="topbar-right"><span id="save-status">Local settings</span><span class="user-chip">XP</span></div></header>
      <div class="page-content">${body}</div>
      <footer class="footer"><span>XP ENHANCE <b>·</b> X-PLANE 12 TOOLKIT</span><span id="footer-state">Settings stored locally</span></footer>
    </main>
  </div>`;

  app.querySelectorAll<HTMLElement>("[data-page]").forEach(el => el.addEventListener("click", () => {
    activePage = el.dataset.page ?? "Dashboard";
    render();
  }));
  app.querySelectorAll<HTMLButtonElement>("[data-feature]").forEach(el => el.addEventListener("click", () => {
    const key = el.dataset.feature as keyof Features;
    settings.features[key] = !settings.features[key];
    render();
    queueSave();
  }));
  app.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach(el => el.addEventListener("click", () => {
    const preset = el.dataset.preset ?? "Balanced";
    settings.preset = preset;
    const profiles: Record<string, { realism: number; features: Features }> = {
      Performance: { realism: 30, features: { colour: true, sharpening: false, ambient_occlusion: false, bloom: false, cloud_enhancement: false, anti_aliasing: true, depth_of_field: false } },
      Balanced: { realism: 65, features: { colour: true, sharpening: true, ambient_occlusion: false, bloom: false, cloud_enhancement: true, anti_aliasing: true, depth_of_field: false } },
      Realistic: { realism: 82, features: { colour: true, sharpening: true, ambient_occlusion: true, bloom: false, cloud_enhancement: true, anti_aliasing: true, depth_of_field: false } },
      Cinematic: { realism: 95, features: { colour: true, sharpening: true, ambient_occlusion: true, bloom: true, cloud_enhancement: true, anti_aliasing: true, depth_of_field: true } }
    };
    settings.realism = profiles[preset].realism;
    settings.features = { ...profiles[preset].features };
    render();
    queueSave();
  }));
  app.querySelector<HTMLInputElement>("#realism-range")?.addEventListener("input", event => {
    settings.realism = Number((event.currentTarget as HTMLInputElement).value);
    const chip = app.querySelector(".value-chip");
    if (chip) chip.textContent = settings.realism + "%";
    queueSave();
  });
  app.querySelector<HTMLButtonElement>("#save-path")?.addEventListener("click", () => {
    const input = app.querySelector<HTMLInputElement>("#xplane-path");
    settings.xplane_path = input?.value.trim() || null;
    queueSave();
    render();
  });
  app.querySelector<HTMLInputElement>("#reshade-file")?.addEventListener("change", async event => {
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    try {
      settings.reshade_ini = await file.text();
      queueSave();
      render();
    } catch {
      setStatus("Could not read the selected INI file");
    }
  });
}

async function start(): Promise<void> {
  try {
    settings = await invoke<Settings>("get_settings");
  } catch (error) {
    console.error("Unable to load settings", error);
    settings = structuredClone(defaults);
  }
  render();
}

void start();