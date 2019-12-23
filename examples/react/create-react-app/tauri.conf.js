const path = require('path')
const distDir = path.resolve(__dirname, './build')

module.exports = function () {
  return {
    build: {
      distDir: distDir,
      devPath: 'http://localhost:3000' // devServer URL or html dir
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
      },
      edge: {
        active: true
      },
      automaticStart: {
        active: true
      }
    }
  }
}
