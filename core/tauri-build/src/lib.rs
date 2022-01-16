// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub use anyhow::Result;

use std::path::{Path, PathBuf};

#[cfg(feature = "codegen")]
mod codegen;

#[cfg(feature = "codegen")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "codegen")))]
pub use codegen::context::CodegenContext;

/// Attributes used on Windows.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WindowsAttributes {
  window_icon_path: PathBuf,
  /// The path to the sdk location. This can be a absolute or relative path. If not supplied
  /// this defaults to whatever `winres` crate determines is the best. See the
  /// [winres documentation](https://docs.rs/winres/*/winres/struct.WindowsResource.html#method.set_toolkit_path)
  sdk_dir: Option<PathBuf>,
}

impl Default for WindowsAttributes {
  fn default() -> Self {
    Self {
      window_icon_path: PathBuf::from("icons/icon.ico"),
      sdk_dir: None,
    }
  }
}

impl WindowsAttributes {
  /// Creates the default attribute set.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the icon to use on the window. Currently only used on Windows.
  /// It must be in `ico` format. Defaults to `icons/icon.ico`.
  pub fn window_icon_path<P: AsRef<Path>>(mut self, window_icon_path: P) -> Self {
    self.window_icon_path = window_icon_path.as_ref().into();
    self
  }

  /// Sets the sdk dir for windows. Currently only used on Windows. This must be a vaild UTF-8
  /// path. Defaults to whatever the `winres` crate determines is best.
  pub fn sdk_dir<P: AsRef<Path>>(mut self, sdk_dir: P) -> Self {
    self.sdk_dir = Some(sdk_dir.as_ref().into());
    self
  }
}

/// The attributes used on the build.
#[derive(Debug, Default)]
pub struct Attributes {
  #[allow(dead_code)]
  windows_attributes: WindowsAttributes,
}

impl Attributes {
  /// Creates the default attribute set.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the icon to use on the window. Currently only used on Windows.
  pub fn windows_attributes(mut self, windows_attributes: WindowsAttributes) -> Self {
    self.windows_attributes = windows_attributes;
    self
  }
}

/// Run all build time helpers for your Tauri Application.
///
/// The current helpers include the following:
/// * Generates a Windows Resource file when targeting Windows.
///
/// # Platforms
///
/// [`build()`] should be called inside of `build.rs` regardless of the platform:
/// * New helpers may target more platforms in the future.
/// * Platform specific code is handled by the helpers automatically.
/// * A build script is required in order to activate some cargo environmental variables that are
///   used when generating code and embedding assets - so [`build()`] may as well be called.
///
/// In short, this is saying don't put the call to [`build()`] behind a `#[cfg(windows)]`.
///
/// # Panics
///
/// If any of the build time helpers fail, they will [`std::panic!`] with the related error message.
/// This is typically desirable when running inside a build script; see [`try_build`] for no panics.
pub fn build() {
  if let Err(error) = try_build(Attributes::default()) {
    panic!("error found during tauri-build: {}", error);
  }
}

/// Non-panicking [`build()`].
#[allow(unused_variables)]
pub fn try_build(attributes: Attributes) -> Result<()> {
  #[cfg(windows)]
  {
    use anyhow::{anyhow, Context};
    use std::fs::read_to_string;
    use tauri_utils::config::Config;
    use winres::WindowsResource;

    let config: Config = serde_json::from_str(
      &read_to_string("tauri.conf.json").expect("failed to read tauri.conf.json"),
    )
    .expect("failed to parse tauri.conf.json");

    let icon_path_string = attributes
      .windows_attributes
      .window_icon_path
      .to_string_lossy()
      .into_owned();

    if attributes.windows_attributes.window_icon_path.exists() {
      let mut res = WindowsResource::new();
      if let Some(sdk_dir) = &attributes.windows_attributes.sdk_dir {
        if let Some(sdk_dir_str) = sdk_dir.to_str() {
          res.set_toolkit_path(sdk_dir_str);
        } else {
          return Err(anyhow!(
            "sdk_dir path is not valid; only UTF-8 characters are allowed"
          ));
        }
      }
      if let Some(version) = &config.package.version {
        res.set("FileVersion", version);
        res.set("ProductVersion", version);
      }
      if let Some(product_name) = &config.package.product_name {
        res.set("ProductName", product_name);
        res.set("FileDescription", product_name);
      }
      res.set_icon_with_id(&icon_path_string, "32512");
      res.compile().with_context(|| {
        format!(
          "failed to compile `{}` into a Windows Resource file during tauri-build",
          icon_path_string
        )
      })?;
    } else {
      return Err(anyhow!(format!(
        "`{}` not found; required for generating a Windows Resource file during tauri-build",
        icon_path_string
      )));
    }
  }

  Ok(())
}
