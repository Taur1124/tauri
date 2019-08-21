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
  tauri = new Runner(appPaths),
  tauriConfig = require('../helpers/tauri-config')({
    ctx: {
      debug: argv.debug
    }
  })
 
require('../generator').generate(tauriConfig.tauri)
require('../entry').generate(appPaths.tauriDir, tauriConfig, true)

require('../helpers/generator')(tauriConfig)
tauri.build(tauriConfig)
