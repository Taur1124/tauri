import { invoke, transformCallback } from './tauri'

/**
 * register a global shortcut
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. Alt+Q
 * @param handler shortcut handler callback
 */
async function registerShortcut(
  shortcut: string,
  handler: () => void
): Promise<void> {
  return invoke({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'register',
      shortcut,
      handler: transformCallback(handler)
    }
  })
}

/**
 * unregister a global shortcut
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. Alt+Q
 */
async function unregisterShortcut(shortcut: string): Promise<void> {
  return invoke({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregister',
      shortcut
    }
  })
}

export { registerShortcut, unregisterShortcut }
