import { TauriConfig } from './../types/config';
const os = require('os')
const spawn = require('cross-spawn').sync
const chalk = require('chalk')
const path = require('path')
const fs = require('fs')

interface DirInfo {
  path: string
  name: string
  type?: 'folder'|'file'
  children?: DirInfo[]
}

function dirTree(filename: string): DirInfo {
  const stats = fs.lstatSync(filename)
  const info: DirInfo = {
    path: filename,
    name: path.basename(filename)
  }

  if (stats.isDirectory()) {
    info.type = 'folder'
    info.children = fs.readdirSync(filename).map(function (child: string) {
      return dirTree(filename + '/' + child)
    });
  } else {
    info.type = 'file'
  }

  return info
}

function getVersion (command: string, args: string[] = [], formatter?: (output: string) => string) {
  try {
    const child = spawn(command, [...args, '--version'])
    if (child.status === 0) {
      const output = String(child.output[1])
      return chalk.green(formatter === undefined ? output : formatter(output)).replace('\n', '')
    }
    return chalk.red('Not installed')
  }
  catch (err) {
    return chalk.red('Not installed')
  }
}

interface Info {
  section?: boolean
  key: string
  value?: string
}

function printInfo (info: Info) {
  console.log(`${info.section ? '\n' : ''}${info.key}${info.value === undefined ? '' : ' - ' + info.value}`)
}

function printAppInfo(tauriDir: string) {
  printInfo({ key: 'App', section: true })

  try {
    const toml = require('@tauri-apps/toml')
    const tomlPath = path.join(tauriDir, 'Cargo.toml')
    const tomlFile = fs.readFileSync(tomlPath)
    // @ts-ignore
    const tomlContents = toml.parse(tomlFile)

    const tauriVersion = () => {
      const tauri = tomlContents.dependencies.tauri
      if (tauri) {
        if (tauri.version) {
          return chalk.green(tauri.version)
        }
        if (tauri.path) {
          try {
            const tauriTomlPath = path.resolve(tauriDir, tauri.path, 'Cargo.toml')
            const tauriTomlFile = fs.readFileSync(tauriTomlPath)
            // @ts-ignore
            const tauriTomlContents = toml.parse(tauriTomlFile)
            return chalk.green(`${tauriTomlContents.package.version} (from source)`)
          } catch (_) { }
        }
      }
      return chalk.red('unknown')
    }

    
    printInfo({ key: '  tauri', value: tauriVersion() })
  }
  catch (_) { }

  try {
    const tauriMode = (config: TauriConfig): string => {
      if (config.tauri.embeddedServer) {
        return chalk.green(config.tauri.embeddedServer.active ? 'embedded-server' : 'no-server')
      }
      return chalk.red('unset')
    }
    const configPath = path.join(tauriDir, 'tauri.conf.json')
    const config = __non_webpack_require__(configPath) as TauriConfig
    printInfo({ key: '  mode', value: tauriMode(config) })
    printInfo({ key: '  build-type', value: config.tauri.bundle && config.tauri.bundle.active ? 'bundle' : 'build' })
    printInfo({ key: '  CSP', value: config.tauri.security ? config.tauri.security.csp : 'unset' })
    printInfo({ key: '  Windows', value: config.tauri.edge && config.tauri.edge.active ? 'Edge' : 'MSHTML' })
    printInfo({ key: '  distDir', value: config.build ? chalk.green(config.build.distDir) : chalk.red('unset') })
    printInfo({ key: '  devPath', value: config.build ? chalk.green(config.build.devPath) : chalk.red('unset') })
  } catch (_) { }
}

module.exports = () => {
  printInfo({ key: 'Operating System', value: chalk.green(`${os.type()}(${os.release()}) - ${os.platform()}/${os.arch()}`), section: true })
  printInfo({ key: 'Node.js environment', section: true })
  printInfo({ key: '  Node.js', value: chalk.green(process.version.slice(1)) })
  printInfo({ key: '  tauri.js', value: chalk.green(require('../../package.json').version) })
  printInfo({ key: 'Rust environment', section: true })
  printInfo({ key: '  rustc', value: getVersion('rustc', [], output => output.split(' ')[1]) })
  printInfo({ key: '  cargo', value: getVersion('cargo', [], output => output.split(' ')[1]) })
  printInfo({ key: '  tauri-cli', value: getVersion('cargo', ['tauri-cli']) })
  printInfo({ key: 'Global packages', section: true })
  printInfo({ key: '  NPM', value: getVersion('npm') })
  printInfo({ key: '  yarn', value: getVersion('yarn') })
  printInfo({ key: 'App directory structure', section: true })

  const { appDir, tauriDir } = require('../helpers/app-paths')
  const tree = dirTree(appDir)
  for (const artifact of (tree.children || [])) {
    if (artifact.type === 'folder') {
      console.log(`/${artifact.name}`)
    }
  }
  printAppInfo(tauriDir)
}
