// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Native system dialogs for opening and saving files.
 *
 * This package is also accessible with `window.__TAURI__.dialog` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "dialog": {
 *         "all": true, // enable all dialog APIs
 *         "open": true, // enable file open API
 *         "save": true // enable file save API
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

/** Extension filters for the file dialog. */
interface DialogFilter {
  /** Filter name. */
  name: string
  /**
   * Extensions to filter, without a `.` prefix.
   * @example
   * ```typescript
   * extensions: ['svg', 'png']
   * ```
   */
  extensions: string[]
}

/** Options for the open dialog. */
interface OpenDialogOptions {
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /** Initial directory or file path. It must exist. */
  defaultPath?: string
  /** Whether the dialog allows multiple selection or not. */
  multiple?: boolean
  /** Whether the dialog is a directory selection or not. */
  directory?: boolean
}

/** Options for the save dialog. */
interface SaveDialogOptions {
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /**
   * Initial directory or file path.
   * If it's a directory path, the dialog interface will change to that folder.
   * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
   */
  defaultPath?: string
}

/**
 * Open a file/directory selection dialog
 *
 * @returns A promise resolving to the selected path(s)
 */
async function open(
  options: OpenDialogOptions = {}
): Promise<string | string[]> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'openDialog',
      options
    }
  })
}

/**
 * Open a file/directory save dialog.
 *
 * @returns A promise resolving to the selected path.
 */
async function save(options: SaveDialogOptions = {}): Promise<string> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'saveDialog',
      options
    }
  })
}

/**
 * Shows a message dialog with an `Ok` button.
 *
 * @param {string} message The message to show.
 *
 * @return {Promise<void>} A promise indicating the success or failure of the operation.
 */
async function message(message: string): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'messageDialog',
      message
    }
  })
}

/**
 * Shows a question dialog with `Yes` and `No` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|undefined} title The dialog's title. Defaults to the application name.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Yes` was clicked or not.
 */
async function ask(message: string, title?: string): Promise<boolean> {
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'askDialog',
      title,
      message
    }
  })
}

/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|undefined} title The dialog's title. Defaults to the application name.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Ok` was clicked or not.
 */
async function confirm(message: string, title?: string): Promise<boolean> {
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'confirmDialog',
      title,
      message
    }
  })
}

export type { DialogFilter, OpenDialogOptions, SaveDialogOptions }

export { open, save, message, ask, confirm }
