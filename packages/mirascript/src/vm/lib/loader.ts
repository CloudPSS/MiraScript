import { VmFunction, type VmImmutable } from '../types/index.js';
import { VmSharedContext } from '../types/context.js';

import type { VmLib } from './helpers.js';
import * as global from './global/index.js';

for (const [name, value] of Object.entries(global)) {
    if (typeof value == 'function') {
        const f = value as VmLib;
        if (f.name !== name) {
            // 如果函数名和导出名不一致，则重命名
            Object.defineProperty(f, 'name', {
                value: name,
                configurable: true,
            });
        }
        VmSharedContext[name] = VmFunction(f, {
            isLib: true,
            injectCp: true,
            fullName: `global.${name}`,
            summary: f.summary,
            params: f.params,
            paramsType: f.paramsType,
            returns: f.returns,
            returnsType: f.returnsType,
        });
    } else {
        VmSharedContext[name] = value as VmImmutable;
    }
}

export const lib = {
    global,
};
