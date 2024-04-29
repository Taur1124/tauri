// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  Manager, Runtime, Webview,
};

use super::ResourceId;

#[command(root = "crate")]
fn close<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<()> {
  webview.resources_table().close(rid)
}

pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("resources")
    .invoke_handler(crate::generate_handler![close])
    .build()
}
