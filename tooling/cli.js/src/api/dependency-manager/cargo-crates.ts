// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawnSync } from './../../helpers/spawn'
import {
  CargoManifest,
  CargoManifestDependency,
  CargoLock
} from './../../types/cargo'
import { ManagementType, Result } from './types'
import { getCrateLatestVersion, semverLt } from './util'
import logger from '../../helpers/logger'
import { resolve as appResolve, tauriDir } from '../../helpers/app-paths'
import { readFileSync, writeFileSync, existsSync } from 'fs'
import inquirer from 'inquirer'
import { createRequire } from 'module'

const require = createRequire(import.meta.url)
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires
const toml = require('@tauri-apps/toml')
const log = logger('dependency:crates')

const dependencies = ['tauri']

function readToml<T>(tomlPath: string): T | null {
  if (existsSync(tomlPath)) {
    const manifest = readFileSync(tomlPath).toString()
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    return toml.parse(manifest) as T
  }
  return null
}

function dependencyDefinition(
  dependency: string | CargoManifestDependency,
  version: string
): string | CargoManifestDependency {
  if (typeof dependency === 'string') {
    return version
  }
  return { ...dependency, version }
}

async function manageDependencies(
  managementType: ManagementType
): Promise<Result> {
  const installedDeps = []
  const updatedDeps = []
  const result: Result = new Map<ManagementType, string[]>()

  const manifest = readToml<CargoManifest>(appResolve.tauri('Cargo.toml'))

  if (manifest === null) {
    log('Cargo.toml not found. Skipping crates check...')
    return result
  }

  const lockPath = appResolve.tauri('Cargo.lock')
  if (!existsSync(lockPath)) {
    spawnSync('cargo', ['generate-lockfile'], tauriDir)
  }
  const lock = readToml<CargoLock>(lockPath)

  for (const dependency of dependencies) {
    const lockPackages = lock
      ? lock.package.filter((pkg) => pkg.name === dependency)
      : []
    // eslint-disable-next-line security/detect-object-injection
    const manifestDep = manifest.dependencies[dependency]
    const currentVersion =
      lockPackages.length === 1
        ? lockPackages[0].version
        : typeof manifestDep === 'string'
        ? manifestDep
        : manifestDep?.version
    if (currentVersion === undefined) {
      log(`Installing ${dependency}...`)
      const latestVersion = getCrateLatestVersion(dependency)
      if (latestVersion !== null) {
        // eslint-disable-next-line security/detect-object-injection
        manifest.dependencies[dependency] = dependencyDefinition(
          // eslint-disable-next-line security/detect-object-injection
          manifest.dependencies[dependency],
          latestVersion
        )
      }
      installedDeps.push(dependency)
    } else if (managementType === ManagementType.Update) {
      const latestVersion = getCrateLatestVersion(dependency)
      if (latestVersion !== null) {
        if (semverLt(currentVersion, latestVersion)) {
          const inquired = (await inquirer.prompt([
            {
              type: 'confirm',
              name: 'answer',
              message: `[CRATES] "${dependency}" latest version is ${latestVersion}. Do you want to update?`,
              default: false
            }
          ])) as { answer: boolean }
          if (inquired.answer) {
            log(`Updating ${dependency}...`)
            // eslint-disable-next-line security/detect-object-injection
            manifest.dependencies[dependency] = dependencyDefinition(
              // eslint-disable-next-line security/detect-object-injection
              manifest.dependencies[dependency],
              latestVersion
            )
            updatedDeps.push(dependency)
          }
        } else {
          // force update the manifest to the show the latest version even if the lockfile is up to date
          // eslint-disable-next-line security/detect-object-injection
          manifest.dependencies[dependency] = dependencyDefinition(
            // eslint-disable-next-line security/detect-object-injection
            manifest.dependencies[dependency],
            latestVersion
          )
          updatedDeps.push(dependency)
          log(`"${dependency}" is up to date`)
        }
      }
    } else {
      log(`"${dependency}" is already installed`)
    }
  }

  if (installedDeps.length || updatedDeps.length) {
    writeFileSync(
      appResolve.tauri('Cargo.toml'),
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-argument
      toml.stringify(manifest as any)
    )
  }
  if (updatedDeps.length) {
    if (!existsSync(appResolve.tauri('Cargo.lock'))) {
      spawnSync('cargo', ['generate-lockfile'], tauriDir)
    }
    spawnSync(
      'cargo',
      [
        'update',
        '--aggressive',
        ...updatedDeps.reduce<string[]>(
          (initialValue, dep) => [...initialValue, '-p', dep],
          []
        )
      ],
      tauriDir
    )
  }

  result.set(ManagementType.Install, installedDeps)
  result.set(ManagementType.Update, updatedDeps)

  return result
}

async function install(): Promise<Result> {
  return await manageDependencies(ManagementType.Install)
}

async function update(): Promise<Result> {
  return await manageDependencies(ManagementType.Update)
}

export { install, update }
