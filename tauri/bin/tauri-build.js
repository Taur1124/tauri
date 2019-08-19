const
  parseArgs = require('minimist'),
  { writeFileSync } = require('fs-extra'),
  path = require('path')

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    d: 'debug'
  },
  boolean: ['h', 'd']
})

if (argv.help) {
  console.log(`
  Description
    Tauri build.
  Usage
    $ tauri build
  Options
    --help, -h     Displays this message
  `)
  process.exit(0)
}

const appPaths = require('../helpers/app-paths'),
  Runner = require('../runner'),
  Injector = require('../injector'),
  tauri = new Runner(appPaths),
  injector = new Injector(appPaths),
  tauriConfig = require('../helpers/tauri-config')({
    ctx: {
      debug: argv.debug
    }
  })
const {bundle, ...cfg} = tauriConfig.tauri,
  cfgDir = injector.configDir()
writeFileSync(path.join(cfgDir, 'config.json'), JSON.stringify(cfg))
writeFileSync(path.join(cfgDir, 'bundle.json'), JSON.stringify(bundle))

require('../helpers/generator')(tauriConfig)
tauri.build(tauriConfig)
