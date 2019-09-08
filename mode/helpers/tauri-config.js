const appPaths = require('./app-paths'),
  merge = require('webpack-merge')

module.exports = cfg => {
  const tauriConf = require(appPaths.resolve.app('tauri.conf.js'))(cfg.ctx)
  const config = merge({
    build: {
      distDir: './dist'
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
        title: require(appPaths.resolve.app('package.json')).productName
      },
      security: {
        csp: 'default-src data: filesystem: ws: http: https: \'unsafe-eval\' \'unsafe-inline\''
      }
    }
  }, tauriConf, cfg)

  process.env.TAURI_DIST_DIR = appPaths.resolve.app(config.build.distDir)
  return config
}
