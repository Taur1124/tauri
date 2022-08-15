// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::Target;
use crate::helpers::{app_paths::tauri_dir, template::JsonMap};
use crate::Result;
use cargo_mobile::{
  android,
  config::{
    self,
    metadata::{self, Metadata},
    Config,
  },
  dot_cargo,
  init::{DOT_FIRST_INIT_CONTENTS, DOT_FIRST_INIT_FILE_NAME},
  opts,
  os::code_command,
  util::{
    self,
    cli::{Report, TextWrapper},
  },
};
use clap::Parser;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};

use std::{
  fs, io,
  path::{Path, PathBuf},
};

use opts::{NonInteractive, OpenInEditor, ReinstallDeps, SkipDevTools};

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri Android project")]
pub struct Options {
  /// Skip prompting for values
  #[clap(long)]
  ci: bool,
}

pub fn command(mut options: Options, target: Target) -> Result<()> {
  options.ci = options.ci || std::env::var("CI").is_ok();

  let wrapper = TextWrapper::with_splitter(textwrap::termwidth(), textwrap::NoHyphenation);
  exec(
    target,
    &wrapper,
    options.ci.into(),
    SkipDevTools::No,
    ReinstallDeps::Yes,
    OpenInEditor::No,
    tauri_dir(),
  )
  .map_err(|e| anyhow::anyhow!("{:#}", e))?;
  Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  ConfigLoadOrGen(config::LoadOrGenError),
  #[error("failed to init first init file {path}: {cause}")]
  DotFirstInitWrite { path: PathBuf, cause: io::Error },
  #[error("failed to create asset dir {asset_dir}: {cause}")]
  AssetDirCreation {
    asset_dir: PathBuf,
    cause: io::Error,
  },
  #[error("failed to install LLDB VS Code extension: {0}")]
  LldbExtensionInstall(bossy::Error),
  #[error(transparent)]
  DotCargoLoad(dot_cargo::LoadError),
  #[error(transparent)]
  HostTargetTripleDetection(util::HostTargetTripleError),
  #[error(transparent)]
  Metadata(metadata::Error),
  #[cfg(target_os = "macos")]
  #[error(transparent)]
  IosInit(super::ios::project::Error),
  #[error(transparent)]
  AndroidEnv(android::env::Error),
  #[error(transparent)]
  AndroidInit(super::android::project::Error),
  #[error(transparent)]
  DotCargoWrite(dot_cargo::WriteError),
  #[error("failed to delete first init file {path}: {cause}")]
  DotFirstInitDelete { path: PathBuf, cause: io::Error },
  #[error(transparent)]
  OpenInEditor(util::OpenInEditorError),
}

pub fn exec(
  target: Target,
  wrapper: &TextWrapper,
  non_interactive: NonInteractive,
  skip_dev_tools: SkipDevTools,
  #[allow(unused_variables)] reinstall_deps: ReinstallDeps,
  open_in_editor: OpenInEditor,
  cwd: impl AsRef<Path>,
) -> Result<Config, Error> {
  let cwd = cwd.as_ref();
  let (config, config_origin) =
    Config::load_or_gen(cwd, non_interactive, wrapper).map_err(Error::ConfigLoadOrGen)?;
  let dot_first_init_path = config.app().root_dir().join(DOT_FIRST_INIT_FILE_NAME);
  let dot_first_init_exists = {
    let dot_first_init_exists = dot_first_init_path.exists();
    if config_origin.freshly_minted() && !dot_first_init_exists {
      // indicate first init is ongoing, so that if we error out and exit
      // the next init will know to still use `WildWest` filtering
      log::info!("creating first init dot file at {:?}", dot_first_init_path);
      fs::write(&dot_first_init_path, DOT_FIRST_INIT_CONTENTS).map_err(|cause| {
        Error::DotFirstInitWrite {
          path: dot_first_init_path.clone(),
          cause,
        }
      })?;
      true
    } else {
      dot_first_init_exists
    }
  };

  let asset_dir = config.app().asset_dir();
  if !asset_dir.is_dir() {
    fs::create_dir_all(&asset_dir).map_err(|cause| Error::AssetDirCreation { asset_dir, cause })?;
  }
  if skip_dev_tools.no() && util::command_present("code").unwrap_or_default() {
    let mut command = code_command();
    command.add_args(&["--install-extension", "vadimcn.vscode-lldb"]);
    if non_interactive.yes() {
      command.add_arg("--force");
    }
    command
      .run_and_wait()
      .map_err(Error::LldbExtensionInstall)?;
  }
  let mut dot_cargo = dot_cargo::DotCargo::load(config.app()).map_err(Error::DotCargoLoad)?;
  // Mysteriously, builds that don't specify `--target` seem to fight over
  // the build cache with builds that use `--target`! This means that
  // alternating between i.e. `cargo run` and `cargo apple run` would
  // result in clean builds being made each time you switched... which is
  // pretty nightmarish. Specifying `build.target` in `.cargo/config`
  // fortunately has the same effect as specifying `--target`, so now we can
  // `cargo run` with peace of mind!
  //
  // This behavior could be explained here:
  // https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags
  dot_cargo
    .set_default_target(util::host_target_triple().map_err(Error::HostTargetTripleDetection)?);

  let metadata = Metadata::load(config.app().root_dir()).map_err(Error::Metadata)?;

  // Generate Xcode project
  #[cfg(target_os = "macos")]
  if target == Target::Ios && metadata.apple().supported() {
    super::ios::project::gen(
      config.apple(),
      metadata.apple(),
      handlebars(&config),
      wrapper,
      non_interactive,
      skip_dev_tools,
      reinstall_deps,
    )
    .map_err(Error::IosInit)?;
  } else {
    println!("Skipping iOS init, since it's marked as unsupported in your Cargo.toml metadata");
  }

  // Generate Android Studio project
  if target == Target::Android && metadata.android().supported() {
    match android::env::Env::new() {
      Ok(env) => super::android::project::gen(
        config.android(),
        metadata.android(),
        &env,
        handlebars(&config),
        wrapper,
        &mut dot_cargo,
      )
      .map_err(Error::AndroidInit)?,
      Err(err) => {
        if err.sdk_or_ndk_issue() {
          Report::action_request(
            " to initialize Android environment; Android support won't be usable until you fix the issue below and re-run `cargo mobile init`!",
            err,
          )
          .print(wrapper);
        } else {
          return Err(Error::AndroidEnv(err));
        }
      }
    }
  } else {
    println!("Skipping Android init, since it's marked as unsupported in your Cargo.toml metadata");
  }

  dot_cargo
    .write(config.app())
    .map_err(Error::DotCargoWrite)?;
  if dot_first_init_exists {
    log::info!("deleting first init dot file at {:?}", dot_first_init_path);
    fs::remove_file(&dot_first_init_path).map_err(|cause| Error::DotFirstInitDelete {
      path: dot_first_init_path,
      cause,
    })?;
  }
  Report::victory(
    "Project generated successfully!",
    "Make cool apps! 🌻 🐕 🎉",
  )
  .print(wrapper);
  if open_in_editor.yes() {
    util::open_in_editor(cwd).map_err(Error::OpenInEditor)?;
  }
  Ok(config)
}

fn handlebars(config: &Config) -> (Handlebars<'static>, JsonMap) {
  let mut h = Handlebars::new();
  h.register_escape_fn(handlebars::no_escape);

  h.register_helper("html-escape", Box::new(html_escape));
  h.register_helper("join", Box::new(join));
  h.register_helper("quote-and-join", Box::new(quote_and_join));
  h.register_helper(
    "quote-and-join-colon-prefix",
    Box::new(quote_and_join_colon_prefix),
  );
  h.register_helper("snake-case", Box::new(snake_case));
  h.register_helper("reverse-domain", Box::new(reverse_domain));
  h.register_helper(
    "reverse-domain-snake-case",
    Box::new(reverse_domain_snake_case),
  );
  // don't mix these up or very bad things will happen to all of us
  h.register_helper("prefix-path", Box::new(prefix_path));
  h.register_helper("unprefix-path", Box::new(unprefix_path));

  let mut map = JsonMap::default();
  map.insert("app", config.app());
  #[cfg(target_os = "macos")]
  map.insert("apple", config.apple());
  map.insert("android", config.android());

  (h, map)
}

fn get_str<'a>(helper: &'a Helper) -> &'a str {
  helper
    .param(0)
    .and_then(|v| v.value().as_str())
    .unwrap_or("")
}

fn get_str_array<'a>(
  helper: &'a Helper,
  formatter: impl Fn(&str) -> String,
) -> Option<Vec<String>> {
  helper.param(0).and_then(|v| {
    v.value().as_array().and_then(|arr| {
      arr
        .iter()
        .map(|val| {
          val.as_str().map(
            #[allow(clippy::redundant_closure)]
            |s| formatter(s),
          )
        })
        .collect()
    })
  })
}

fn html_escape(
  helper: &Helper,
  _: &Handlebars,
  _ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&handlebars::html_escape(get_str(helper)))
    .map_err(Into::into)
}

fn join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| s.to_string())
        .ok_or_else(|| RenderError::new("`join` helper wasn't given an array"))?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{:?}", s))
        .ok_or_else(|| RenderError::new("`quote-and-join` helper wasn't given an array"))?
        .join(", "),
    )
    .map_err(Into::into)
}

fn quote_and_join_colon_prefix(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      &get_str_array(helper, |s| format!("{:?}", format!(":{}", s)))
        .ok_or_else(|| {
          RenderError::new("`quote-and-join-colon-prefix` helper wasn't given an array")
        })?
        .join(", "),
    )
    .map_err(Into::into)
}

fn snake_case(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  use heck::ToSnekCase as _;
  out
    .write(&get_str(helper).to_snek_case())
    .map_err(Into::into)
}

fn reverse_domain(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(&util::reverse_domain(get_str(helper)))
    .map_err(Into::into)
}

fn reverse_domain_snake_case(
  helper: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  use heck::ToSnekCase as _;
  out
    .write(&util::reverse_domain(get_str(helper)).to_snek_case())
    .map_err(Into::into)
}

fn app_root(ctx: &Context) -> Result<&str, RenderError> {
  let app_root = ctx
    .data()
    .get("app")
    .ok_or_else(|| RenderError::new("`app` missing from template data."))?
    .get("root-dir")
    .ok_or_else(|| RenderError::new("`app.root-dir` missing from template data."))?;
  app_root
    .as_str()
    .ok_or_else(|| RenderError::new("`app.root-dir` contained invalid UTF-8."))
}

fn prefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::prefix_path(app_root(ctx)?, get_str(helper))
        .to_str()
        .ok_or_else(|| {
          RenderError::new(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.",
          )
        })?,
    )
    .map_err(Into::into)
}

fn unprefix_path(
  helper: &Helper,
  _: &Handlebars,
  ctx: &Context,
  _: &mut RenderContext,
  out: &mut dyn Output,
) -> HelperResult {
  out
    .write(
      util::unprefix_path(app_root(ctx)?, get_str(helper))
        .map_err(|_| {
          RenderError::new("Attempted to unprefix a path that wasn't in the app root dir.")
        })?
        .to_str()
        .ok_or_else(|| {
          RenderError::new(
            "Either the `app.root-dir` or the specified path contained invalid UTF-8.",
          )
        })?,
    )
    .map_err(Into::into)
}
