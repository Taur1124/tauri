// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;

use std::{borrow::Cow, sync::Arc};

use crate::{Pattern, WindowBuilder};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{Config, PatternKind, TauriConfig, WindowUrl},
};

pub struct NoopAsset {
  csp_hashes: Vec<CspHash<'static>>,
}

impl Assets for NoopAsset {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }

  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(self.csp_hashes.iter().copied())
  }
}

pub fn noop_assets() -> NoopAsset {
  NoopAsset {
    csp_hashes: Default::default(),
  }
}

pub fn mock_context<A: Assets>(assets: A) -> crate::Context<A> {
  crate::Context {
    config: Config {
      schema: None,
      package: Default::default(),
      tauri: TauriConfig {
        pattern: PatternKind::Brownfield,
        windows: Vec::new(),
        bundle: Default::default(),
        security: Default::default(),
        system_tray: None,
        macos_private_api: false,
      },
      build: Default::default(),
      plugins: Default::default(),
    },
    assets: Arc::new(assets),
    default_window_icon: None,
    app_icon: None,
    #[cfg(desktop)]
    system_tray_icon: None,
    package_info: crate::PackageInfo {
      name: "test".into(),
      version: "0.1.0".parse().unwrap(),
      authors: "Tauri",
      description: "Tauri test",
      crate_name: "test",
    },
    _info_plist: (),
    pattern: Pattern::Brownfield(std::marker::PhantomData),
  }
}

pub fn mock_app() -> crate::App<MockRuntime> {
  let app = crate::Builder::<MockRuntime>::new()
    .build(mock_context(noop_assets()))
    .unwrap();

  WindowBuilder::new(&app, "main", WindowUrl::App("index.html".into()))
    .build()
    .unwrap();

  app
}
