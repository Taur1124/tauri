// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod rust;

use std::path::Path;

use crate::helpers::{config::Config, manifest::Manifest};
use tauri_bundler::bundle::{PackageType, Settings, SettingsBuilder};

#[allow(clippy::too_many_arguments)]
pub fn get_bundler_settings(
  app_settings: rust::AppSettings,
  target: String,
  features: &[String],
  manifest: &Manifest,
  config: &Config,
  out_dir: &Path,
  package_types: Option<Vec<PackageType>>,
) -> crate::Result<Settings> {
  let mut settings_builder = SettingsBuilder::new()
    .package_settings(app_settings.get_package_settings())
    .bundle_settings(app_settings.get_bundle_settings(config, manifest, features)?)
    .binaries(app_settings.get_binaries(config, &target)?)
    .project_out_directory(out_dir)
    .target(target);

  if let Some(types) = package_types {
    settings_builder = settings_builder.package_types(types);
  }

  settings_builder.build().map_err(Into::into)
}
