// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { join } from 'path'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const svelte: Recipe = {
  descriptiveName: {
    name: 'Svelte (https://github.com/sveltejs/template)',
    value: 'svelte'
  },
  shortName: 'svelte',
  extraNpmDevDependencies: [],
  extraNpmDependencies: [],
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'confirm',
        name: 'typescript',
        message: 'Enable Typescript?',
        default: true,
        loop: false,
        when: !ci
      }
    ]
  },
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../public`,
    devPath: 'http://localhost:8080',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } dev`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  preInit: async ({ cwd, cfg, answers, ci }) => {
    let typescript = false
    if (answers) {
      typescript = !!answers.typescript
    }

    await shell(
      'npx',
      [ci ? '--yes' : '', 'degit', 'sveltejs/template', `${cfg.appName}`],
      {
        cwd
      }
    )

    if (typescript) {
      await shell('node', ['scripts/setupTypeScript.js'], {
        cwd: join(cwd, cfg.appName)
      })
    }
  },
  postInit: async ({ cfg, packageManager }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager} install
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)

    return await Promise.resolve()
  }
}

export { svelte }
