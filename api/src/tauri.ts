declare global {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface Window {
    __TAURI_INVOKE_HANDLER__: (command: { [key: string]: unknown }) => void
  }
}

function s4(): string {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

function uid(): string {
  return (
    s4() +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    s4() +
    s4()
  )
}

function transformCallback(
  callback?: (response: any) => void,
  once = false
): string {
  const identifier = uid()

  Object.defineProperty(window, identifier, {
    value: (result: any) => {
      if (once) {
        Reflect.deleteProperty(window, identifier)
      }

      return callback?.(result)
    },
    writable: false,
    configurable: true
  })

  return identifier
}

/**
 * sends a message to the backend
 *
 * @param args
 *
 * @return {Promise<T>} Promise resolving or rejecting to the backend response
 */
async function invoke<T>(
  cmd: string | { [key: string]: unknown },
  args: { [key: string]: unknown } = {}
): Promise<T> {
  return new Promise((resolve, reject) => {
    const callback = transformCallback((e) => {
      resolve(e)
      Reflect.deleteProperty(window, error)
    }, true)
    const error = transformCallback((e) => {
      reject(e)
      Reflect.deleteProperty(window, callback)
    }, true)

    if (typeof cmd === 'string') {
      args.cmd = cmd
    } else if (typeof cmd === 'object') {
      args = cmd
    } else {
      return reject(new Error('Invalid argument type.'))
    }

    window.__TAURI_INVOKE_HANDLER__({
      callback,
      error,
      ...args
    })
  })
}

export { transformCallback, invoke }
