---
"tauri-utils": "patch:bug"
---

Fixed an issue where configuration parsing errors always displayed 'tauri.conf.json' as the file path, even when using 'Tauri.toml' or 'tauri.conf.json5'. 

The error messages now correctly shows the actual config file being used.
