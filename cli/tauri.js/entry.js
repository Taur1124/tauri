const compileTemplate = require('lodash.template')
const { readFileSync, writeFileSync, ensureDir } = require('fs-extra')
const path = require('path')

module.exports.generate = (outDir, cfg) => {
  // this MUST be from the templates repo
  const apiTemplate = readFileSync(path.resolve(__dirname, '../lib/tauri.js'), 'utf-8')
  const apiContent = compileTemplate(apiTemplate)({
    ...cfg,
    confName: 'tauri.conf.js'
  })
  ensureDir(outDir).then(() => {
    writeFileSync(path.join(outDir, 'tauri.js'), apiContent, 'utf-8')
  })
}
