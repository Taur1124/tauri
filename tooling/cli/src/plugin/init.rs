// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use crate::{
  helpers::{resolve_tauri_path, template},
  VersionMetadata,
};
use anyhow::Context;
use clap::Parser;
use dialoguer::Input;
use handlebars::{to_json, Handlebars};
use heck::{AsKebabCase, ToKebabCase, ToSnakeCase};
use include_dir::{include_dir, Dir};
use log::warn;
use std::{
  collections::BTreeMap,
  env::current_dir,
  ffi::OsStr,
  fmt::Display,
  fs::{create_dir_all, remove_dir_all, File, OpenOptions},
  path::{Path, PathBuf},
  str::FromStr,
};

const BACKEND_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/backend");
const API_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/with-api");
const ANDROID_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/android");
const IOS_PLUGIN_DIR: Dir<'_> = include_dir!("templates/plugin/ios");

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri plugin project")]
pub struct Options {
  /// Name of your Tauri plugin
  #[clap(short = 'n', long = "name")]
  plugin_name: String,
  /// Initializes a Tauri plugin with TypeScript API
  #[clap(long)]
  api: bool,
  /// Initializes a Tauri core plugin (internal usage)
  #[clap(long, hide(true))]
  tauri: bool,
  /// Set target directory for init
  #[clap(short, long)]
  #[clap(default_value_t = current_dir().expect("failed to read cwd").display().to_string())]
  directory: String,
  /// Path of the Tauri project to use (relative to the cwd)
  #[clap(short, long)]
  tauri_path: Option<PathBuf>,
  /// Author name
  #[clap(short, long)]
  author: Option<String>,
  /// Adds native Android support.
  #[clap(long)]
  android: bool,
  /// Adds native iOS support.
  #[clap(long)]
  ios: bool,
}

impl Options {
  fn load(&mut self) {
    if self.author.is_none() {
      self.author.replace(if self.tauri {
        "Tauri Programme within The Commons Conservancy".into()
      } else {
        "You".into()
      });
    }
  }
}

pub fn command(mut options: Options) -> Result<()> {
  options.load();
  let template_target_path = PathBuf::from(options.directory).join(format!(
    "tauri-plugin-{}",
    AsKebabCase(&options.plugin_name)
  ));
  let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../../metadata.json"))?;
  if template_target_path.exists() {
    warn!("Plugin dir ({:?}) not empty.", template_target_path);
  } else {
    let (tauri_dep, tauri_example_dep, tauri_build_dep) =
      if let Some(tauri_path) = options.tauri_path {
        (
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            r#"{{  path = {:?} }}"#,
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri-build")
          ),
        )
      } else {
        (
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri),
          format!(r#"{{ version = "{}" }}"#, metadata.tauri_build),
        )
      };

    let _ = remove_dir_all(&template_target_path);
    let handlebars = Handlebars::new();

    let mut data = BTreeMap::new();
    data.insert("plugin_name_original", to_json(&options.plugin_name));
    data.insert("plugin_name", to_json(options.plugin_name.to_kebab_case()));
    data.insert(
      "plugin_name_snake_case",
      to_json(options.plugin_name.to_snake_case()),
    );
    data.insert("tauri_dep", to_json(tauri_dep));
    data.insert("tauri_example_dep", to_json(tauri_example_dep));
    data.insert("tauri_build_dep", to_json(tauri_build_dep));
    data.insert("author", to_json(options.author));

    if options.tauri {
      data.insert(
        "license_header",
        to_json(
          "// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
             // SPDX-License-Identifier: Apache-2.0
             // SPDX-License-Identifier: MIT\n\n"
            .replace("  ", "")
            .replace(" //", "//"),
        ),
      );
    }

    template::render(
      &handlebars,
      &data,
      if options.api {
        &API_PLUGIN_DIR
      } else {
        &BACKEND_PLUGIN_DIR
      },
      &template_target_path,
    )
    .with_context(|| "failed to render Tauri template")?;

    if options.android {
      let plugin_id = request_input(
        "What should be the Package ID for your plugin?",
        Some(format!("com.plugin.{}", options.plugin_name)),
        false,
        false,
      )?
      .unwrap();

      let mut data = BTreeMap::new();
      data.insert("package_id", to_json(&plugin_id));

      let mut created_dirs = Vec::new();
      let dest = template_target_path.join("android");
      template::render_with_generator(
        &handlebars,
        &data,
        &ANDROID_PLUGIN_DIR,
        &dest,
        &mut |path| {
          generate_android_out_file(path, &dest, &plugin_id.replace('.', "/"), &mut created_dirs)
        },
      )
      .with_context(|| "failed to render plugin Android template")?;
    }

    if options.ios {
      template::render(
        &handlebars,
        &data,
        &IOS_PLUGIN_DIR,
        template_target_path.join("ios"),
      )
      .with_context(|| "failed to render plugin iOS template")?;
    }
  }
  Ok(())
}

fn generate_android_out_file(
  path: &Path,
  dest: &Path,
  package_path: &str,
  created_dirs: &mut Vec<PathBuf>,
) -> std::io::Result<Option<File>> {
  let mut iter = path.iter();
  let root = iter.next().unwrap().to_str().unwrap();
  let path = match (root, path.extension().and_then(|o| o.to_str())) {
    ("src", Some("kt")) => {
      let parent = path.parent().unwrap();
      let file_name = path.file_name().unwrap();
      let out_dir = dest.join(parent).join(package_path);
      out_dir.join(file_name)
    }
    _ => dest.join(path),
  };

  let parent = path.parent().unwrap().to_path_buf();
  if !created_dirs.contains(&parent) {
    create_dir_all(&parent)?;
    created_dirs.push(parent);
  }

  let mut options = OpenOptions::new();
  options.write(true);

  #[cfg(unix)]
  if path.file_name().unwrap() == OsStr::new("gradlew") {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o755);
  }

  if path.file_name().unwrap() == OsStr::new("BuildTask.kt") || !path.exists() {
    options.create(true).open(path).map(Some)
  } else {
    Ok(None)
  }
}

fn request_input<T>(
  prompt: &str,
  initial: Option<T>,
  skip: bool,
  allow_empty: bool,
) -> Result<Option<T>>
where
  T: Clone + FromStr + Display + ToString,
  T::Err: Display + std::fmt::Debug,
{
  if skip {
    Ok(initial)
  } else {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut builder = Input::with_theme(&theme);
    builder.with_prompt(prompt);
    builder.allow_empty(allow_empty);

    if let Some(v) = initial {
      builder.with_initial_text(v.to_string());
    }

    builder.interact_text().map(Some).map_err(Into::into)
  }
}
