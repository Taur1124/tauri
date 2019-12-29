'use strict'

/**
 * This is a module that takes an original image and resizes
 * it to common icon sizes and will put them in a folder.
 * It will retain transparency and can make special file
 * types. You can control the settings.
 *
 * @module tauricon
 * @exports tauricon
 * @author Daniel Thompson-Yvetot
 * @license MIT
 */

import { access, ensureDir, ensureFileSync, writeFileSync } from 'fs-extra'
import imagemin, { Plugin } from 'imagemin'
import optipng from 'imagemin-optipng'
import pngquant from 'imagemin-pngquant'
import zopfli from 'imagemin-zopfli'
import isPng from 'is-png'
import path from 'path'
import * as png2icons from 'png2icons'
import readChunk from 'read-chunk'
import sharp from 'sharp'
import { appDir, tauriDir } from '../helpers/app-paths'
import logger from '../helpers/logger'
import * as settings from '../helpers/tauricon.config'

const log = logger('app:spawn')
const warn = logger('app:spawn', 'red')

let image: boolean | sharp.Sharp = false
const spinnerInterval = false

const exists = async function(file: string | Buffer): Promise<boolean> {
  try {
    await access(file)
    return true
  } catch (err) {
    return false
  }
}

/**
 * This is the first call that attempts to memoize the sharp(src).
 * If the source image cannot be found or if it is not a png, it
 * is a failsafe that will exit or throw.
 *
 * @param {string} src - a folder to target
 * @exits {error} if not a png, if not an image
 */
const checkSrc = async (src: string): Promise<boolean | sharp.Sharp> => {
  if (image !== false) {
    return image
  } else {
    const srcExists = await exists(src)
    if (!srcExists) {
      image = false
      if (spinnerInterval) clearInterval(spinnerInterval)
      warn('[ERROR] Source image for tauricon not found')
      process.exit(1)
    } else {
      const buffer = await readChunk(src, 0, 8)
      if (isPng(buffer) === true) {
        return (image = sharp(src))
      } else {
        image = false
        if (spinnerInterval) clearInterval(spinnerInterval)
        warn('[ERROR] Source image for tauricon is not a png')
        process.exit(1)
      }
    }
  }
}

/**
 * Sort the folders in the current job for unique folders.
 *
 * @param {object} options - a subset of the settings
 * @returns {array} folders
 */
// TODO: proper type of options and folders
const uniqueFolders = (options: { [index: string]: any }): any[] => {
  let folders = []
  for (const type in options) {
    const option = options[String(type)]
    if (option.folder) {
      folders.push(option.folder)
    }
  }
  // TODO: is compare argument required?
  // eslint-disable-next-line @typescript-eslint/require-array-sort-compare
  folders = folders.sort().filter((x, i, a) => !i || x !== a[i - 1])
  return folders
}

/**
 * Turn a hex color (like #212342) into r,g,b values
 *
 * @param {string} hex - hex colour
 * @returns {array} r,g,b
 */
const hexToRgb = (
  hex: string
): { r: number, g: number, b: number } | undefined => {
  // https://stackoverflow.com/questions/5623838/rgb-to-hex-and-hex-to-rgb
  // Expand shorthand form (e.g. "03F") to full form (e.g. "0033FF")
  const shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i
  hex = hex.replace(shorthandRegex, function(
    m: string,
    r: string,
    g: string,
    b: string
  ) {
    return r + r + g + g + b + b
  })

  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    }
    : undefined
}

/**
 * validate image and directory
 */
const validate = async (
  src: string,
  target: string
): Promise<boolean | sharp.Sharp> => {
  if (target !== undefined) {
    await ensureDir(target)
  }
  return checkSrc(src)
}

// TODO: should take end param?
/**
 * Log progress in the command line
 *
 * @param {boolean} end
 */
const progress = (msg: string): void => {
  process.stdout.write(`  ${msg}                       \r`)
}

/**
 * Create a spinner on the command line
 *
 * @example
 *
 *     const spinnerInterval = spinner()
 *     // later
 *     clearInterval(spinnerInterval)
 */
const spinner = (): NodeJS.Timeout => {
  return setInterval(() => {
    process.stdout.write('/ \r')
    setTimeout(() => {
      process.stdout.write('- \r')
      setTimeout(() => {
        process.stdout.write('\\ \r')
        setTimeout(() => {
          process.stdout.write('| \r')
        }, 100)
      }, 100)
    }, 100)
  }, 500)
}

const tauricon = (exports.tauricon = {
  validate: async function(src: string, target: string) {
    await validate(src, target)
    return typeof image === 'object'
  },
  version: function() {
    return __non_webpack_require__('../../package.json').version
  },
  make: async function(
    src: string = path.resolve(appDir, 'app-icon.png'),
    target: string = path.resolve(tauriDir, 'icons'),
    strategy: string,
    // TODO: proper type for options
    options: { [index: string]: any }
  ) {
    const spinnerInterval = spinner()
    options = options || settings.options.tauri
    await this.validate(src, target)
    progress('Building Tauri icns and ico')
    await this.icns(src, target, options, strategy)
    progress('Building Tauri png icons')
    await this.build(src, target, options)
    if (strategy) {
      progress(`Minifying assets with ${strategy}`)
      await this.minify(target, options, strategy, 'batch')
    } else {
      log('no minify strategy')
    }
    progress('Tauricon Finished')
    clearInterval(spinnerInterval)
    return true
  },

  /**
   * Creates a set of images according to the subset of options it knows about.
   *
   * @param {string} src - image location
   * @param {string} target - where to drop the images
   * @param {object} options - js object that defines path and sizes
   */
  build: async function(
    src: string,
    target: string,
    // TODO: proper type for options
    options: { [index: string]: any }
  ) {
    await this.validate(src, target)
    const sharpSrc = sharp(src) // creates the image object
    const buildify2 = async function(
      pvar: [string, number, number]
    ): Promise<void> {
      try {
        const pngImage = sharpSrc.resize(pvar[1], pvar[1])
        if (pvar[2]) {
          const rgb = hexToRgb(options.background_color) || {
            r: undefined,
            g: undefined,
            b: undefined
          }
          pngImage.flatten({
            background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 }
          })
        }
        pngImage.png()
        await pngImage.toFile(pvar[0])
      } catch (err) {
        warn(err)
      }
    }

    let output
    const folders = uniqueFolders(options)
    // eslint-disable-next-line @typescript-eslint/no-for-in-array
    for (const n in folders) {
      const folder = folders[Number(n)]
      // make the folders first
      // TODO: should this be ensureDirSync?
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      ensureDir(`${target}${path.sep}${folder}`)
    }
    for (const optionKey in options) {
      const option = options[String(optionKey)]
      // chain up the transforms
      for (const sizeKey in option.sizes) {
        const size = option.sizes[String(sizeKey)]
        if (!option.splash) {
          const dest = `${target}/${option.folder}`
          if (option.infix === true) {
            output = `${dest}${path.sep}${option.prefix}${size}x${size}${option.suffix}`
          } else {
            output = `${dest}${path.sep}${option.prefix}${option.suffix}`
          }
          const pvar: [string, number, number] = [
            output,
            size,
            option.background
          ]
          await buildify2(pvar)
        }
      }
    }
  },
  /**
   * Creates a set of splash images (COMING SOON!!!)
   *
   * @param {string} src - icon location
   * @param {string} splashSrc - splashscreen location
   * @param {string} target - where to drop the images
   * @param {object} options - js object that defines path and sizes
   */
  splash: async function(
    src: string,
    splashSrc: string,
    target: string,
    // TODO: proper type for options
    options: { [index: string]: any }
  ) {
    let output
    let block = false
    const rgb = hexToRgb(options.background_color) || {
      r: undefined,
      g: undefined,
      b: undefined
    }

    // three options
    // options: splashscreen_type [generate | overlay | pure]
    //          - generate (icon + background color) DEFAULT
    //          - overlay (icon + splashscreen)
    //          - pure (only splashscreen)

    let sharpSrc
    if (splashSrc === src) {
      // prevent overlay or pure
      block = true
    }
    if (block === true || options.splashscreen_type === 'generate') {
      await this.validate(src, target)
      if (!image) {
        process.exit(1)
      }
      sharpSrc = sharp(src)
      sharpSrc
        .extend({
          top: 726,
          bottom: 726,
          left: 726,
          right: 726,
          background: {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            alpha: 1
          }
        })
        .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
    } else if (options.splashscreen_type === 'overlay') {
      sharpSrc = sharp(splashSrc)
        .flatten({ background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 } })
        .composite([
          {
            input: src
            // blend: 'multiply' <= future work, maybe just a gag
          }
        ])
    } else if (options.splashscreen_type === 'pure') {
      sharpSrc = sharp(splashSrc).flatten({
        background: { r: rgb.r, g: rgb.g, b: rgb.b, alpha: 1 }
      })
    }
    // TODO: determine if this really could be undefined
    // @ts-ignore
    const data = await sharpSrc.toBuffer()

    for (const optionKey in options) {
      const option = options[String(optionKey)]
      for (const sizeKey in option.sizes) {
        const size = option.sizes[String(sizeKey)]
        if (option.splash) {
          const dest = `${target}${path.sep}${option.folder}`
          await ensureDir(dest)

          if (option.infix === true) {
            output = `${dest}${path.sep}${option.prefix}${size}x${size}${option.suffix}`
          } else {
            output = `${dest}${path.sep}${option.prefix}${option.suffix}`
          }
          const pvar = [output, size]
          let sharpData = sharp(data)
          sharpData = sharpData.resize(pvar[1][0], pvar[1][1])
          await sharpData.toFile(pvar[0])
        }
      }
    }
  },

  /**
   * Minifies a set of images
   *
   * @param {string} target - image location
   * @param {object} options - where to drop the images
   * @param {string} strategy - which minify strategy to use
   * @param {string} mode - singlefile or batch
   */
  minify: async function(
    target: string,
    // TODO: proper type for options
    options: { [index: string]: any },
    strategy: string,
    mode: string
  ) {
    let cmd: Plugin
    const minify = settings.options.minify
    if (!minify.available.find(x => x === strategy)) {
      strategy = minify.type
    }
    switch (strategy) {
      case 'pngquant':
        // TODO: is minify.pngquantOptions the proper format?
        // @ts-ignore
        cmd = pngquant(minify.pngquantOptions)
        break
      case 'optipng':
        cmd = optipng(minify.optipngOptions)
        break
      case 'zopfli':
        cmd = zopfli(minify.zopfliOptions)
        break
    }

    const __minifier = async (pvar: string[]): Promise<string | void> => {
      await imagemin([pvar[0]], {
        destination: pvar[1],
        plugins: [cmd]
      }).catch(err => {
        warn(err)
      })
    }
    switch (mode) {
      case 'singlefile':
        // TODO: the __minifier function only accepts one arg, why is cmd passed?
        // @ts-ignore
        await __minifier([target, path.dirname(target)], cmd)
        break
      case 'batch':
        // eslint-disable-next-line no-case-declarations
        const folders = uniqueFolders(options)
        // eslint-disable-next-line @typescript-eslint/no-for-in-array
        for (const n in folders) {
          const folder = folders[Number(n)]
          // TODO: The log argument doesn't accept multiple args, should this be fixed?
          // @ts-ignore
          log('batch minify:', folder)
          await __minifier(
            [
              `${target}${path.sep}${folder}${path.sep}*.png`,
              `${target}${path.sep}${folder}`
            ],
            // TODO: the __minifier function only accepts one arg, why is this here?
            // @ts-ignore
            cmd
          )
        }
        break
      default:
        warn('[ERROR] Minify mode must be one of [ singlefile | batch]')
        process.exit(1)
    }
    return 'minified'
  },

  /**
   * Creates special icns and ico filetypes
   *
   * @param {string} src - image location
   * @param {string} target - where to drop the images
   * @param {object} options
   * @param {string} strategy
   */
  icns: async function(
    src: string,
    target: string,
    // TODO: proper type for options
    options: { [index: string]: any },
    strategy: string
  ) {
    try {
      if (!image) {
        process.exit(1)
      }
      await this.validate(src, target)

      const sharpSrc = sharp(src)
      const buf = await sharpSrc.toBuffer()

      const out = png2icons.createICNS(buf, png2icons.BICUBIC, 0)
      ensureFileSync(path.join(target, '/icon.icns'))
      writeFileSync(path.join(target, '/icon.icns'), out)

      const out2 = png2icons.createICO(buf, png2icons.BICUBIC, 0, true)
      ensureFileSync(path.join(target, '/icon.ico'))
      writeFileSync(path.join(target, '/icon.ico'), out2)
    } catch (err) {
      console.error(err)
      throw err
    }
  }
})

if (typeof exports !== 'undefined') {
  if (typeof module !== 'undefined' && module.exports) {
    exports = module.exports = tauricon
  }
  exports.tauricon = tauricon
}
