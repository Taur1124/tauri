#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[cfg(feature = "api")]
pub enum Cmd {
  #[cfg(any(feature = "all-api", feature = "readTextFile"))]
  ReadTextFile {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "readBinaryFile"))]
  ReadBinaryFile {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "writeFile"))]
  WriteFile {
    file: String,
    contents: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "listFiles"))]
  ListFiles {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "listDirs"))]
  ListDirs {
    path: String,
    callback: String,
    error: String,
  },
  #[cfg(any(feature = "all-api", feature = "setTitle"))]
  SetTitle { title: String },
  #[cfg(any(feature = "all-api", feature = "execute"))]
  Execute {
    command: String,
    args: Vec<String>,
    callback: String,
    error: String,
  },
}
