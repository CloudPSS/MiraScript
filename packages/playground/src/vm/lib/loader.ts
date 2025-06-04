import { VmFunction, type VmImmutable } from '../types/index.js';
import { VmSharedGlobal } from '../types/global';

import * as global from './global.js';
import type { VmFunctionLike } from '../types/function.js';

for (const [name, value] of Object.entries(global)) {
    if (typeof value == 'function') {
        VmSharedGlobal[name] = VmFunction(value as VmFunctionLike, { isLib: true, fullName: `global.${name}` });
    } else {
        VmSharedGlobal[name] = value as VmImmutable;
    }
}
