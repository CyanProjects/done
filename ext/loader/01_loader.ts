import {
    op_loader_cache_keys,
    op_loader_cache_delete
} from "ext:core/ops";

class DoneLoaderAPI {
    constructor() {}

    keys() {
        return op_loader_cache_keys()
    }

    drop(key: string) {
        return op_loader_cache_delete(key)
    }
}

const DoneLoader = new DoneLoaderAPI()

export { DoneLoader }
