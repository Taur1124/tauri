// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::{Command, Parser};
use clap_complete::{generate, Shell};
use log::info;
use std::io::{Cursor, Write};

const PKG_MANAGERS: &[&str] = &["cargo", "pnpm", "npm", "yarn"];

#[derive(Debug, Clone, Parser)]
#[clap(about = "Shell completions")]
pub struct Options {
  /// Shell to generate a completion script for.
  #[clap(short, long, verbatim_doc_comment)]
  shell: Shell,
}

fn commands_for_completions(shell: Shell, cmd: Command) -> Vec<Command> {
  if matches!(shell, Shell::Zsh | Shell::PowerShell | Shell::Fish) {
    let tauri = cmd.name("tauri");
    PKG_MANAGERS
      .iter()
      .map(|manager| {
        if manager == &"npm" {
          Command::new(manager).subcommand(Command::new("run").subcommand(tauri.clone()))
        } else {
          Command::new(manager).subcommand(tauri.clone())
        }
      })
      .collect()
  } else {
    vec![Command::new("cargo").subcommand(cmd.name("tauri"))]
  }
}

fn print_completions(shell: Shell, cmd: Command) -> Result<()> {
  let mut buffer = Cursor::new(Vec::new());
  for (i, mut cmd) in commands_for_completions(shell, cmd).into_iter().enumerate() {
    let bin_name = cmd
      .get_bin_name()
      .map(|s| s.to_string())
      .unwrap_or_else(|| cmd.get_name().to_string());

    let mut buf = Vec::new();
    generate(shell, &mut cmd, &bin_name, &mut buf);

    let completions = if shell == Shell::PowerShell {
      let s = String::from_utf8_lossy(&buf);
      if i != 0 {
        // namespaces have already been imported
        s.replace("using namespace System.Management.Automation.Language", "")
          .replace("using namespace System.Management.Automation", "")
          .as_bytes()
          .to_vec()
      } else {
        s.as_bytes().to_vec()
      }
    } else {
      buf
    };

    buffer.write_all(&completions)?;
  }

  let b = buffer.into_inner();
  let mut completions = String::from_utf8_lossy(&b).into_owned();

  for manager in PKG_MANAGERS {
    match shell {
      Shell::Bash => completions.push_str(&format!(
        "complete -F _cargo -o bashdefault -o default {} tauri\n",
        if manager == &"npm" {
          "npm run"
        } else {
          manager
        }
      )),
      Shell::Fish => {}
      Shell::Zsh => {}
      Shell::PowerShell => {}
      _ => {}
    };
  }

  print!("{}", completions);

  Ok(())
}

pub fn command(options: Options, cmd: Command) -> Result<()> {
  info!("Generating completion file for {}...", options.shell);

  print_completions(options.shell, cmd)?;

  Ok(())
}
