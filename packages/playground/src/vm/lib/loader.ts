import { VmFunction } from '../types';
import { VmSharedGlobal } from '../types/global';

import * as global from './global.js';

for (const [name, value] of Object.entries(global)) {
    VmSharedGlobal[name] = VmFunction(value, { isLib: true, fullName: `global.${name}` });
}
