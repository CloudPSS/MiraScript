import { kVmScript, VM_SCRIPT_NAME } from '../helpers/constants.js';
import { defineProperty, entries } from '../helpers/utils.js';
import * as operations from '../vm/operations/index.js';
import type { VmValue, VmContext } from '../vm/types/index.js';
import type { InputMode, ScriptInput } from './types.js';

/** Mirascript 脚本 */
export type VmScriptLike = (global?: VmContext) => VmValue;

/** Mirascript 脚本 */
export type VmScript = VmScriptLike & {
    readonly [kVmScript]: InputMode;
    /** 原始代码 */
    readonly source: string;
};

/** 生成 JS 函数 */
export function wrapScript(source: ScriptInput, mode: InputMode, script: VmScriptLike): VmScript {
    /* c8 ignore next 3 */
    if (kVmScript in script) {
        return script as VmScriptLike as VmScript;
    }
    defineProperty(script, 'name', { value: VM_SCRIPT_NAME });
    defineProperty(script, kVmScript, { value: mode });
    if (typeof source === 'string') {
        defineProperty(script, 'source', { value: source });
    } else if (source instanceof Uint8Array) {
        defineProperty(script, 'source', { value: '<buffer>' });
    }
    return script as VmScript;
}

const [keys, values] = (() => {
    const keys: string[] = [];
    const values: unknown[] = [];
    for (const [key, value] of entries(operations)) {
        keys.push(key);
        values.push(value);
    }
    return [keys.join(','), values] as const;
})();

/** 生成 JS 函数 */
export function createScript(source: ScriptInput, mode: InputMode, code: string): VmScript {
    let script;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        script = new Function(keys, code)(...values) as VmScriptLike;
        /* c8 ignore next 3 */
    } catch (error) {
        throw new Error(`Failed to create script`, { cause: error });
    }
    return wrapScript(source, mode, script);
}
