import { PackageManager } from '../dependency-manager'
import { shell } from '../shell'
import { Recipe } from '../types/recipe'

const addAdditionalPackage = async (
  packageManager: PackageManager,
  cwd: string,
  appName: string,
  packageName: string
): Promise<void> => {
  const ngCommand = ['ng', 'add', packageName, '--skip-confirmation']

  if (packageManager === 'yarn') {
    await shell('yarn', ngCommand, {
      cwd: `${cwd}/${appName}`
    })
  } else {
    await shell('npm', ['run', ...ngCommand], {
      cwd: `${cwd}/${appName}`
    })
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
    beforeDevCommand: `${packageManager === 'yarn' ? 'yarn' : 'npm run'} start`,
    beforeBuildCommand: `${
      packageManager === 'yarn' ? 'yarn' : 'npm run'
    } build`
  }),
  extraQuestions: ({ ci }) => {
    return [
      {
        type: 'confirm',
        name: 'material',
        message: 'Add Angular Material (https://material.angular.io/)?',
        validate: (input: string) => {
          return input.toLowerCase() === 'yes'
        },
        loop: false,
        when: !ci
      },
      {
        type: 'confirm',
        name: 'eslint',
        message:
          'Add Angular ESLint (https://github.com/angular-eslint/angular-eslint)?',
        validate: (input: string) => {
          return input.toLowerCase() === 'yes'
        },
        loop: false,
        when: !ci
      }
    ]
  },
  preInit: async ({ cwd, cfg, answers, packageManager }) => {
    // Angular CLI creates the folder for you
    await shell(
      'npx',
      [
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

    if (answers) {
      if (answers.material) {
        await addAdditionalPackage(
          packageManager,
          cwd,
          cfg.appName,
          '@angular/material'
        )
      }

      if (answers.eslint) {
        await addAdditionalPackage(
          packageManager,
          cwd,
          cfg.appName,
          '@angular-eslint/schematics'
        )
      }
    }
  },
  postInit: async ({ packageManager, cfg }) => {
    console.log(`
      Your installation completed.

      $ cd ${cfg.appName}
      $ ${packageManager === 'yarn' ? 'yarn' : 'npm run'} tauri ${
      packageManager === 'npm' ? '--' : ''
    } dev
    `)

    return await Promise.resolve()
  }
}

export { ngcli }
