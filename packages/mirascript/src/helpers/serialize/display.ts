import type { VmAny, VmExtern, VmFunction, VmModule } from '../../vm/index.js';
import { rethrowControl } from '../../vm/effects/state.js';
import { getVmFunctionInfo } from '../types/index.js';
import type { SerializeOptions } from './interface.js';
import { DEFAULT_OPTIONS, mergeOptions, serializeImpl } from './serialize.js';

/** 将 MiraScript 函数转化为 MiraScript 字符串 */
export function displayFunction(value: VmFunction): string {
    try {
        const name = getVmFunctionInfo(value)?.fullName;
        return name ? `<function ${name}>` : `<function>`;
        /* c8 ignore next 4 */
    } catch (error) {
        rethrowControl(error);
        return `<function>`;
    }
}

/** 将 MiraScript 模块转化为 MiraScript 字符串 */
export function displayModule(value: VmModule): string {
    try {
        return value.toString(true);
        /* c8 ignore next 4 */
    } catch (error) {
        rethrowControl(error);
        return `<module>`;
    }
}

/** 将 MiraScript 外部值转化为 MiraScript 字符串 */
export function displayExtern(value: VmExtern): string {
    try {
        const tag = `<extern ${value.tag}>`;
        const rep = value.toString(true);
        if (rep === tag || rep.length > 50) {
            return tag;
        }
        return `${tag} ${rep}`;
        /* c8 ignore next 4 */
    } catch (error) {
        rethrowControl(error);
        return `<extern>`;
    }
}

export const DISPLAY_OPTIONS = Object.freeze({
    ...DEFAULT_OPTIONS,
    maxDepth: 3,
    serializeFunction: displayFunction,
    serializeModule: displayModule,
    serializeExtern: displayExtern,
} satisfies SerializeOptions);

/**
 * 将 MiraScript 值转化为 MiraScript 字符串。
 */
export function display(value: VmAny, options?: Partial<SerializeOptions>): string {
    const opt = mergeOptions(DISPLAY_OPTIONS, options);
    return serializeImpl(value, 0, opt);
}
