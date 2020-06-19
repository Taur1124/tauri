import tauri from './tauri'
import { BaseDirectory, FsOptions, FsFileOption, FileEntry } from './models'

/**
 * reads a file as text
 *
 * @param filePath path to the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function readTextFile(filePath: string, options: FsOptions = {}): Promise<string> {
  return tauri.readTextFile(filePath, options)
}

/**
 * reads a file as binary
 *
 * @param filePath path to the file
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<int[]>}
 */
async function readBinaryFile(filePath: string, options: FsOptions = {}): Promise<string> {
  return tauri.readBinaryFile(filePath, options)
}

/**
 * writes a text file
 *
 * @param file
 * @param file.path path of the file
 * @param file.contents contents of the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function writeFile(file: FsFileOption, options: FsOptions = {}): Promise<void> {
  return tauri.writeFile(file, options)
}

/**
 * list directory files
 *
 * @param dir path to the directory to read
 * @param [options] configuration object
 * @param [options.recursive] whether to list dirs recursively or not
 * @param [options.dir] base directory
 * @return
 */
async function readDir(dir: string, options: FsOptions = {}): Promise<FileEntry[]> {
  return tauri.readDir(dir, options)
}

/**
 * Creates a directory
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to create
 * @param [options] configuration object
 * @param [options.recursive] whether to create the directory's parent components or not
 * @param [options.dir] base directory
 * @return
 */
async function createDir(dir: string, options: FsOptions = {}): Promise<void> {
  return tauri.createDir(dir, options)
}

/**
 * Removes a directory
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to remove
 * @param [options] configuration object
 * @param [options.recursive] whether to remove all of the directory's content or not
 * @param [options.dir] base directory
 * @return
 */
async function removeDir(dir: string, options: FsOptions = {}): Promise<void> {
  return tauri.removeDir(dir, options)
}

/**
 * Copy file
 *
 * @param source
 * @param destination
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function copyFile(source: string, destination: string, options: FsOptions = {}): Promise<void> {
  return tauri.copyFile(source, destination, options)
}

/**
 * Removes a file
 *
 * @param file path to the file to remove
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function removeFile(file: string, options: FsOptions = {}): Promise<void> {
  return tauri.removeFile(file, options)
}

/**
 * Renames a file
 *
 * @param oldPath
 * @param newPath
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function renameFile(oldPath: string, newPath: string, options: FsOptions = {}): Promise<void> {
  return tauri.renameFile(oldPath, newPath, options)
}

export {
  BaseDirectory as Dir,
  readTextFile,
  readBinaryFile,
  writeFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile,
  renameFile
}
