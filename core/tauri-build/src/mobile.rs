use std::{
  env::{var, var_os},
  fs::{self, rename},
  path::{Path, PathBuf},
};

use anyhow::Result;

#[derive(Default)]
pub struct PluginBuilder {
  android_path: Option<PathBuf>,
  ios_path: Option<PathBuf>,
}

impl PluginBuilder {
  /// Creates a new builder for mobile plugin functionality.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets the Android project path.
  pub fn android_path<P: Into<PathBuf>>(mut self, android_path: P) -> Self {
    self.android_path.replace(android_path.into());
    self
  }

  /// Sets the iOS project path.
  pub fn ios_path<P: Into<PathBuf>>(mut self, ios_path: P) -> Self {
    self.ios_path.replace(ios_path.into());
    self
  }

  /// Injects the mobile templates in the given path relative to the manifest root.
  pub fn run(self) -> Result<()> {
    let target_os = var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
      "android" => {
        if let Some(path) = self.android_path {
          let manifest_dir = var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
          let source = manifest_dir.join(path);

          let tauri_library_path = std::env::var("DEP_TAURI_ANDROID_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_ANDROID_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");

          copy_folder(
            Path::new(&tauri_library_path),
            &source.join("tauri-api"),
            &[],
          )?;

          if let Some(project_dir) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
            let pkg_name = var("CARGO_PKG_NAME").unwrap();
            println!("cargo:rerun-if-env-changed=TAURI_ANDROID_PROJECT_PATH");
            let android_plugin_project_path = project_dir.join("tauri-plugins").join(&pkg_name);

            inject_android_project(&source, android_plugin_project_path, &["tauri-api"])?;

            let gradle_settings_path = project_dir.join("tauri.settings.gradle");
            let gradle_settings = fs::read_to_string(&gradle_settings_path)?;
            let include = format!(
              "include ':{pkg_name}'
project(':{pkg_name}').projectDir = new File('./tauri-plugins/{pkg_name}')"
            );
            if !gradle_settings.contains(&include) {
              fs::write(
                &gradle_settings_path,
                format!("{gradle_settings}\n{include}"),
              )?;
            }

            let app_build_gradle_path = project_dir.join("app").join("tauri.build.gradle.kts");
            let app_build_gradle = fs::read_to_string(&app_build_gradle_path)?;
            let implementation = format!(r#"implementation(project(":{pkg_name}"))"#);
            let target = "dependencies {";
            if !app_build_gradle.contains(&implementation) {
              fs::write(
                &app_build_gradle_path,
                app_build_gradle.replace(target, &format!("{target}\n  {implementation}")),
              )?
            }
          }
        }
      }
      #[cfg(target_os = "macos")]
      "ios" => {
        if let Some(path) = self.ios_path {
          let manifest_dir = var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
          let tauri_library_path = std::env::var("DEP_TAURI_IOS_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_IOS_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");

          copy_folder(
            &Path::new(&tauri_library_path),
            &path.join("tauri-api"),
            &[".build", "Package.resolved", "Tests"],
          )?;
          link_swift_library(&var("CARGO_PKG_NAME").unwrap(), manifest_dir.join(path));
        }
      }
      _ => (),
    }

    Ok(())
  }
}

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub fn link_swift_library(name: &str, source: impl AsRef<Path>) {
  let source = source.as_ref();
  println!("cargo:rerun-if-changed={}", source.display());
  let curr_dir = std::env::current_dir().unwrap();
  std::env::set_current_dir(&source).unwrap();
  swift_rs::build::SwiftLinker::new("10.13")
    .with_ios("11")
    .with_package(name, source)
    .link();
  std::env::set_current_dir(&curr_dir).unwrap();
}

#[doc(hidden)]
pub fn inject_android_project(
  source: impl AsRef<Path>,
  target: impl AsRef<Path>,
  ignore_paths: &[&str],
) -> Result<()> {
  let source = source.as_ref();
  let target = target.as_ref();

  // keep build folder if it exists
  let build_path = target.join("build");
  let out_dir = if build_path.exists() {
    let out_dir = target.parent().unwrap().join(".tauri-tmp-build");
    rename(&build_path, &out_dir)?;
    Some(out_dir)
  } else {
    None
  };

  copy_folder(source, target, ignore_paths)?;

  if let Some(out_dir) = out_dir {
    rename(out_dir, &build_path)?;
  }

  let rerun_path = target.join("build.gradle.kts");
  let metadata = source.join("build.gradle.kts").metadata()?;
  filetime::set_file_mtime(
    &rerun_path,
    filetime::FileTime::from_last_modification_time(&metadata),
  )?;

  println!("cargo:rerun-if-changed={}", rerun_path.display());

  Ok(())
}

fn copy_folder(source: &Path, target: &Path, ignore_paths: &[&str]) -> Result<()> {
  let _ = fs::remove_dir_all(target);

  for entry in walkdir::WalkDir::new(source) {
    let entry = entry?;
    let rel_path = entry.path().strip_prefix(source)?;
    let rel_path_str = rel_path.to_string_lossy();
    if ignore_paths
      .iter()
      .any(|path| rel_path_str.starts_with(path))
    {
      continue;
    }
    let dest_path = target.join(rel_path);

    if entry.file_type().is_dir() {
      fs::create_dir(&dest_path)?;
    } else {
      fs::copy(entry.path(), &dest_path)?;
      println!("cargo:rerun-if-changed={}", entry.path().display());
    }
  }

  Ok(())
}
