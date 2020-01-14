use tempfile;

mod utils;
use ignore::Walk;
use serde::Serialize;
use std::fs;
use std::fs::metadata;
use utils::get_dir_name_from_path;

use tempfile::tempdir;

#[derive(Debug, Serialize)]
pub struct DiskEntry {
  pub path: String,
  pub is_dir: bool,
  pub name: String,
}

fn is_dir(file_name: String) -> crate::Result<bool> {
  match metadata(file_name.to_string()) {
    Ok(md) => return Result::Ok(md.is_dir()),
    Err(err) => return Result::Err(err.to_string().into()),
  };
}

pub fn walk_dir(path_copy: String) -> crate::Result<Vec<DiskEntry>> {
  println!("Trying to walk: {}", path_copy.as_str());
  let mut files_and_dirs: Vec<DiskEntry> = vec![];
  for result in Walk::new(path_copy) {
    match result {
      Ok(entry) => {
        let display_value = entry.path().display();
        let _dir_name = display_value.to_string();

        match is_dir(display_value.to_string()) {
          Ok(flag) => {
            files_and_dirs.push(DiskEntry {
              path: display_value.to_string(),
              is_dir: flag,
              name: display_value.to_string(),
            });
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
  }
  return Result::Ok(files_and_dirs);
}

pub fn list_dir_contents(dir_path: &String) -> crate::Result<Vec<DiskEntry>> {
  fs::read_dir(dir_path)
    .map_err(|err| crate::Error::with_chain(err, "read string failed"))
    .and_then(|paths| {
      let mut dirs: Vec<DiskEntry> = vec![];
      for path in paths {
        let dir_path = path.expect("dirpath error").path();
        let _dir_name = dir_path.display();
        dirs.push(DiskEntry {
          path: format!("{}", _dir_name),
          is_dir: true,
          name: get_dir_name_from_path(_dir_name.to_string()),
        });
      }
      Ok(dirs)
    })
}

pub fn with_temp_dir<F: FnOnce(&tempfile::TempDir) -> ()>(callback: F) -> crate::Result<()> {
  let dir = tempdir()?;
  callback(&dir);
  dir.close()?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use totems::{assert_ok, assert_some};

  // check is dir function by passing in arbitrary strings
  #[quickcheck]
  fn qc_is_dir(f: String) -> bool {
    // is the string runs through is_dir and comes out as an OK result then it must be a DIR.
    match is_dir(f.clone()) {
      // check to see that the path exists.
      Ok(_) => std::path::PathBuf::from(f).exists(),
      // if is Err then string isn't a path nor a dir and function passes.
      Err(_) => true,
    }
  }

  #[test]
  // check the walk_dir function
  fn check_walk_dir() {
    // define a relative directory string test/
    let dir = String::from("test/");
    // add the file to this directory as test/test.txt
    let file = format!("{}test.txt", &dir).to_string();

    // call walk_dir on the directory
    let res = walk_dir(dir.clone());

    // assert that the result is Ok()
    assert_ok!(&res);

    // destruct the OK into a vector of DiskEntry Structs
    if let Ok(vec) = res {
      // assert that the vector length is only 2
      assert_eq!(vec.len(), 2);

      // get the first DiskEntry
      let first = &vec[0];
      // get the second DiskEntry
      let second = &vec[1];

      // check the fields for the first DiskEntry
      assert_eq!(first.path, dir);
      assert_eq!(first.is_dir, true);
      assert_eq!(first.name, dir);

      // check the fields for the second DiskEntry
      assert_eq!(second.path, file);
      assert_eq!(second.is_dir, false);
      assert_eq!(second.name, file);
    }
  }
}
