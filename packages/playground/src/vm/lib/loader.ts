import { VmFunction, type VmImmutable } from '../types/index.js';
import { VmSharedGlobal } from '../types/global';

import * as global from './global.js';
import type { VmFunctionLike } from '../types/function.js';

/** 库函数 */
export type VmLib = VmFunctionLike & {
    summary?: string;
    params?: Record<string, string>;
};

for (const [name, value] of Object.entries(global)) {
    let e = name;
    if (name.startsWith('_') && name.endsWith('_')) {
        e = name.slice(1, -1);
    }
    if (typeof value == 'function') {
        const f = value as VmLib;
        VmSharedGlobal[e] = VmFunction(f, {
            isLib: true,
            injectCp: true,
            fullName: `global.${e}`,
            summary: f.summary,
            params: f.params,
        });
    } else {
        VmSharedGlobal[e] = value as VmImmutable;
    }
}
