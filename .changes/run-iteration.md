---
"tauri": patch
---

Adds `run_iteration` API to the `App` and return the app instance on the `build` method of the `Builder`. The `run_iteration` method runs the window event loop step by step, allowing Tauri to be run along other applications.
