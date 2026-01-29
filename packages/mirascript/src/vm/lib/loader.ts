import {
    isVmModule,
    VmFunction,
    VmModule,
    type VmConst,
    type VmFunctionLike,
    type VmImmutable,
} from '../types/index.js';
import { create, defineProperty, entries } from '../../helpers/utils.js';

import type { VmLib } from './helpers.js';

/** 内置模块 */
class VmBuiltinModule<const T extends Record<string, VmImmutable> = Record<string, VmImmutable>> extends VmModule<T> {
    constructor(
        name: string,
        value: T,
        readonly descriptions: Record<keyof T, string | undefined>,
    ) {
        super(name, value);
    }
    /** @inheritdoc */
    override describe(key: string): string | undefined {
        return this.descriptions[key as keyof T];
    }
}

/** 原始值 */
export type RawValue = VmLib<VmFunctionLike | VmConst> | VmBuiltinModule;
/** 包装值 */
type ToWrappedValue<V extends RawValue> = V extends VmLib<infer C> ? (C extends VmFunctionLike ? VmFunction<C> : C) : V;
/** 包装值 */
export function wrapEntry<const T extends RawValue>(
    name: string,
    value: T,
    module: string,
): [value: ToWrappedValue<T>, description: string | undefined] {
    if (isVmModule(value)) {
        return [value as ToWrappedValue<T>, undefined];
    }
    if (typeof value != 'function') {
        if (value == null || typeof value != 'object' || !('value' in value)) {
            throw new TypeError(`Cannot wrap non-function, non-const value: ${name} in module ${module}`);
        }
        return [value.value as ToWrappedValue<T>, value.summary || undefined];
    }
    if (value.name !== name) {
        // 如果函数名和导出名不一致，则重命名
        defineProperty(value, 'name', {
            value: name,
            configurable: true,
        });
    }
    return [
        VmFunction(value, {
            ...value,
            isLib: true,
            injectCp: false,
            fullName: `${module}.${name}`,
        }) as ToWrappedValue<T>,
        value.summary || undefined,
    ];
}

/** 创建模块 */
export type ToWrappedModule<T extends Record<string, RawValue>> = VmBuiltinModule<{
    [key in keyof T]: ToWrappedValue<T[key]>;
}>;

/** 创建模块 */
export function createModule<const T extends Record<string, RawValue>>(name: string, lib: T): ToWrappedModule<T> {
    const mod = create(null) as Record<string, VmImmutable>;
    const descriptions = create(null) as Record<string, string | undefined>;
    for (const [key, value] of entries(lib)) {
        const [wrappedValue, description] = wrapEntry(key, value, name);
        mod[key] = wrappedValue;
        descriptions[key] = description;
    }
    return new VmBuiltinModule(name, mod, descriptions) as ToWrappedModule<T>;
}
