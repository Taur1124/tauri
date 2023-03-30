// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

window.print = function () {
  return window.__TAURI_INVOKE__('tauri', {
    __tauriModule: 'Window',
    message: {
      cmd: 'manage',
      data: {
        cmd: {
          type: 'print'
        }
      }
    }
  })
}
