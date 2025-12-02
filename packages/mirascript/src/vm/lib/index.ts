import * as global from './global/index.js';
import * as mods from './mod/index.js';
import { VM_SHARED_CONTEXT, VM_SHARED_CONTEXT_DESCRIPTIONS } from '../types/context.js';
import { create, entries } from '../../helpers/utils.js';
import { createModule, wrapEntry, type RawValue } from './loader.js';

for (const [name, value] of entries(global)) {
    const [wrappedValue, description] = wrapEntry(name, value as RawValue, 'global');
    VM_SHARED_CONTEXT[name] = wrappedValue;
    VM_SHARED_CONTEXT_DESCRIPTIONS[name] = description;
}

for (const [name, value] of entries(mods)) {
    const mod = createModule(name, value as Record<string, RawValue>);
    const [wrappedValue, description] = wrapEntry(name, mod, 'global');
    VM_SHARED_CONTEXT[name] = wrappedValue;
    VM_SHARED_CONTEXT_DESCRIPTIONS[name] = description;
}

export const lib: Readonly<typeof global & typeof mods> = Object.freeze(Object.assign(create(null), global, mods));
