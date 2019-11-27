const path = require('path')
const distDir = path.resolve(__dirname, './dist')

module.exports = function () {
  return {
    automaticStart: {
      active: true
    },
    build: {
      distDir: distDir,
      APP_URL: 'http://localhost:4000' // must use a localhost server for now
    },
    ctx: {},
    tauri: {
      embeddedServer: {
        active: true
      },
      bundle: {
        active: true
      },
      whitelist: {
        all: false
      },
      window: {
        title: 'Tauri App'
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      }
    },
    edge: true
  }
}
