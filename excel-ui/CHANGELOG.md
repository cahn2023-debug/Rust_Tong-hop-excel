# Changelog

## [1.0.0] - 2026-04-08
### Added
- Context menu for discovered files with "Mở file Excel" and "Mở thư mục chứa file" actions.
- Functional `open_file` and `open_folder` Tauri commands using `opener` plugin.
- Global click listener to auto-hide the context menu.
- Build system configuration for Windows `.msi` installer.

### Fixed
- Fixed issue where context menu items were unresponsive.
- Fixed context menu remaining visible after clicking elsewhere.
