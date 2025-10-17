import { VmFunction, VmModule, type VmConst, type VmFunctionLike, type VmImmutable } from '../types/index.js';
import { create, defineProperty, entries } from '../../helpers/utils.js';
import { VmSharedContext } from '../types/context.js';

import type { VmLib, VmLibOption } from './_helpers.js';
import * as global from './global/index.js';

for (const [name, value] of entries(global)) {
    VmSharedContext[name] = wrapEntry(name, value as RawValue);
}

/** 原始值 */
type RawValue = VmLib | VmConst | VmModule;
/** 包装值 */
type ToWrappedValue<V extends RawValue> = V extends VmFunctionLike ? VmFunction<V> : V;
/** 包装值 */
function wrapEntry<const T extends RawValue>(name: string, value: T): ToWrappedValue<T> {
    if (typeof value == 'function') {
        if (value.name !== name) {
            // 如果函数名和导出名不一致，则重命名
            defineProperty(value, 'name', {
                value: name,
                configurable: true,
            });
        }
        return VmFunction(value, {
            isLib: true,
            injectCp: true,
            fullName: `global.${name}`,
            ...(value as VmLibOption),
        }) as ToWrappedValue<T>;
    } else {
        return value as ToWrappedValue<T>;
    }
}

/** 创建模块 */
export type ToWrappedModule<T extends Record<string, RawValue>> = VmModule<{
    [key in keyof T]: ToWrappedValue<T[key]>;
}>;

/** 创建模块 */
export function createModule<const T extends Record<string, RawValue>>(name: string, lib: T): ToWrappedModule<T> {
    const mod = create(null) as Record<string, VmImmutable>;
    for (const [key, value] of entries(lib)) {
        mod[key] = wrapEntry(key, value);
    }
    return new VmModule(name, mod) as ToWrappedModule<T>;
}

export const lib = global;
