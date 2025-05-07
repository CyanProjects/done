import { primordials } from 'ext:core/mod.js'

import {
    op_loader_cache_keys,
    op_loader_cache_entries,
    op_loader_cache_delete,
    op_loader_cache_get,
    op_loader_cache_set,
    op_loader_module_get,
    op_loader_module_requests,
    op_loader_resolve,
} from "ext:core/ops";

const { SymbolFor } = primordials

export class LinkedModuleRef {
    #id?: number
    readonly #specifier: string

    constructor(specifier: string, id?: number) {
        this.#id = id
        this.#specifier = specifier
    }

    getId() {
        if (!this.#id) this.#id = op_loader_cache_get(this.#specifier)
        return this.#id!
    }

    refresh() {
        this.#id = op_loader_cache_get(this.specifier)
    }

    get specifier() {
        return this.#specifier
    }

    exports() {
        return op_loader_module_get(this.getId())
    }

    requests() {
        return op_loader_module_requests(this.getId())
    }

    drop() {
        return op_loader_cache_delete(this.specifier)
    }

    [SymbolFor("Deno.privateCustomInspect")]() {
        return `ModuleRef@${this.specifier}#${this.getId()}`
    }
}

export class DoneLoaderAPI {
    constructor() {}

    keys() {
        return op_loader_cache_keys()
    }

    get(key: string) {
        return new LinkedModuleRef(key, op_loader_cache_get(key))
    }

    set(key: string, ref: LinkedModuleRef) {
        const ret = op_loader_cache_set(key, ref.getId());
        if (ret === 0) return
        return ret
    }

    resolve = op_loader_resolve

    entries() {
        const entries: [string, number][] = op_loader_cache_entries()
        return entries
            .map(([specifier, moduleId]) => [specifier, new LinkedModuleRef(specifier, moduleId)] as const)
    }

    drop(key: string) {
        return op_loader_cache_delete(key)
    }
}

export const DoneLoader = new DoneLoaderAPI()
