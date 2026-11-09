# XP Enhance

Windows desktop companion for X-Plane 12.

## Version 0.1.0 — Foundation
- WPF desktop application with dark navigation layout.
- Manual X-Plane 12 folder selection and validation.
- Visual profile controls with saved slider/toggle values.
- Preset selection.
- Settings stored in %APPDATA%\\XPEnhance.
- GitHub Actions Windows x64 build and release workflow.

## Not implemented yet
The foundation build does not modify X-Plane rendering, install ReShade or FlyWithLua, install community addons, or provide DLSS/neural rendering. Visual controls currently save configuration values only.

## Build
Requires Windows and the .NET 10 SDK:
```powershell
dotnet restore XPEnhance.csproj
dotnet run --project XPEnhance.csproj
dotnet publish XPEnhance.csproj -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true -o publish
```

Push to main to trigger GitHub Actions. Download the build from the workflow run's Artifacts section. Push a tag such as v0.1.0 to publish a GitHub Release ZIP.

Future installers must validate archive paths, verify downloads, preserve existing files, back up conflicts and support rollback.