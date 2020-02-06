import crossSpawn from 'cross-spawn'
import logger from './logger'

const log = logger('app:spawn')
const warn = logger('app:spawn', 'red')

/*
  Returns pid, takes onClose
 */
export const spawn = (
  cmd: string,
  params: string[],
  cwd: string,
  onClose: (code: number) => void
): number => {
  log(`Running "${cmd} ${params.join(' ')}"`)
  log()

  // TODO: move to execa?
  const runner = crossSpawn(cmd, params, {
    stdio: 'inherit',
    cwd
  })

  runner.on('close', code => {
    log()
    if (code) {
      // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
      log(`Command "${cmd}" failed with exit code: ${code}`)
    }

    onClose?.(code)
  })

  return runner.pid
}

/*
  Returns nothing, takes onFail
 */
export const spawnSync = (
  cmd: string,
  params: string[],
  cwd: string,
  onFail: () => void
): void => {
  log(`[sync] Running "${cmd} ${params.join(' ')}"`)
  log()

  const runner = crossSpawn.sync(cmd, params, {
    stdio: 'inherit',
    cwd
  })

  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
  if (runner.status || runner.error) {
    warn()
    // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
    warn(`⚠️  Command "${cmd}" failed with exit code: ${runner.status}`)
    if (runner.status === null) {
      warn(`⚠️  Please globally install "${cmd}"`)
    }
    onFail?.()
    process.exit(1)
  }
}
