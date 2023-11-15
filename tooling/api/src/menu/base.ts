// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { Resource } from '../internal'
import { Channel, invoke } from '../primitives'

export type ItemKind =
  | 'MenuItem'
  | 'Predefined'
  | 'Check'
  | 'Icon'
  | 'Submenu'
  | 'Menu'

export async function newMenu(
  kind: ItemKind,
  opts?: unknown
): Promise<[number, string]> {
  const handler = new Channel<string>()
  let items: null | Array<[number, string]> = null
  if (opts && typeof opts === 'object') {
    if ('action' in opts && opts.action) {
      handler.onmessage = opts.action as () => void
      delete opts.action
    }

    if ('items' in opts && opts.items) {
      items = (opts.items as { rid: number; kind: string }[]).map((i) => [
        i.rid,
        i.kind
      ])
    }
  }

  return invoke('plugin:menu|new', {
    kind,
    options: opts ? { ...opts, items } : undefined,
    handler
  })
}

export class MenuItemBase extends Resource {
  /** @ignore */
  readonly #id: string
  /** @ignore */
  readonly #kind: ItemKind

  /** The id of this item. */
  get id(): string {
    return this.#id
  }

  /** @ignore */
  get kind(): string {
    return this.#kind
  }

  /** @ignore */
  protected constructor(rid: number, id: string, kind: ItemKind) {
    super(rid)
    this.#id = id
    this.#kind = kind
  }
}
