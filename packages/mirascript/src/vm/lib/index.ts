import * as global from './global/index.js';
import * as mods from './mod/index.js';
import { VmSharedContext } from '../types/context.js';
import { create, entries } from '../../helpers/utils.js';
import { createModule, wrapEntry, type RawValue } from './loader.js';

for (const [name, value] of entries(global)) {
    VmSharedContext[name] = wrapEntry(name, value as RawValue, 'global');
}

for (const [name, value] of entries(mods)) {
    const mod = createModule(name, value as Record<string, RawValue>);
    VmSharedContext[name] = wrapEntry(name, mod, 'global');
}

export const lib = Object.freeze(Object.assign(create(null), global, mods));
