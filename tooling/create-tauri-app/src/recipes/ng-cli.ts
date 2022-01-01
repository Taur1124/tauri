import { PackageManager } from '../dependency-manager'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'
import { join } from 'path'

const addAdditionalPackage = async (
  packageManager: PackageManager,
  cwd: string,
  appName: string,
  packageName: string
): Promise<void> => {
  const ngCommand = ['ng', 'add', packageName, '--skip-confirmation']

  switch (packageManager) {
    case 'pnpm':
    case 'yarn':
      await shell(packageManager, ngCommand, {
        cwd: join(cwd, appName)
      })
      break

    case 'npm':
      await shell('npm', ['run', ...ngCommand], {
        cwd: join(cwd, appName)
      })
      break
  }
}

const ngcli: Recipe = {
  descriptiveName: {
    name: 'Angular CLI (https://angular.io/cli)',
    value: 'ng-cli'
  },
  shortName: 'ngcli',
  extraNpmDependencies: [],
  extraNpmDevDependencies: [],
  configUpdate: ({ cfg, packageManager }) => ({
    ...cfg,
    distDir: `../dist/${cfg.appName}`,
    devPath: 'http://localhost:4200',
    beforeDevCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } start`,
    beforeBuildCommand: `${
      packageManager === 'npm' ? 'npm run' : packageManager
    } build`
  }),
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'confirm',
        name: 'material',
        message: 'Add Angular Material (https://material.angular.io/)?',
        when: !ci
      },
      {
        type: 'confirm',
        name: 'eslint',
        message:
          'Add Angular ESLint (https://github.com/angular-eslint/angular-eslint)?',
        when: !ci
      }
    ]
  },
  preInit: async ({ cwd, cfg, answers, packageManager, ci }) => {
    await shell(
      'npx',
      [
        ci ? '--yes' : '',
        '-p',
        '@angular/cli',
        'ng',
        'new',
        `${cfg.appName}`,
        `--package-manager=${packageManager}`
      ],
      {
        cwd
      }
    )

    if (answers?.material) {
      await addAdditionalPackage(
        packageManager,
        cwd,
        cfg.appName,
        '@angular/material'
      )
    }

    if (answers?.eslint) {
      await addAdditionalPackage(
        packageManager,
        cwd,
        cfg.appName,
        '@angular-eslint/schematics'
      )
    }
  },
  postInit: async ({ packageManager, cfg }) => {
    console.log(`
    Your installation completed.

    $ cd ${cfg.appName}
    $ ${packageManager === 'npm' ? 'npm run' : packageManager} tauri dev
    `)

    return await Promise.resolve()
  }
}

export { ngcli }
