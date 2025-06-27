import { VmFunction, type VmImmutable } from '../types/index.js';
import { VmSharedContext } from '../types/context.js';

import type { VmLib } from './helpers.js';
import * as global from './global.js';

for (const [name, value] of Object.entries(global)) {
    let e = name;
    if (name.startsWith('_') && name.endsWith('_')) {
        e = name.slice(1, -1);
    }
    if (typeof value == 'function') {
        const f = value as VmLib;
        if (f.name !== e) {
            // 如果函数名和导出名不一致，则重命名
            Object.defineProperty(f, 'name', {
                value: e,
                configurable: true,
            });
        }
        VmSharedContext[e] = VmFunction(f, {
            isLib: true,
            injectCp: true,
            fullName: `global.${e}`,
            summary: f.summary,
            params: f.params,
            paramsType: f.paramsType,
            returns: f.returns,
            returnsType: f.returnsType,
        });
    } else {
        VmSharedContext[e] = value as VmImmutable;
    }
}

export const lib = {
    global,
};
