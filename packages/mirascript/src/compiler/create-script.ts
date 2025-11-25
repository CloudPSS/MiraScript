import { kVmScript, VM_SCRIPT_NAME } from '../helpers/constants.js';
import { defineProperty } from '../helpers/utils.js';
import { keys, values } from '../vm/env.js';
import type { VmValue, VmContext } from '../vm/types/index.js';
import type { ScriptInput } from './types.js';

/** Mirascript 脚本 */
export type VmScriptLike = (global?: VmContext) => VmValue;

/** Mirascript 脚本 */
export type VmScript = VmScriptLike & {
    readonly [kVmScript]: true;
    /** 原始代码 */
    readonly source: string;
};

/** 生成 JS 函数 */
export function wrapScript(source: ScriptInput, script: VmScriptLike): VmScript {
    /* c8 ignore next 3 */
    if (kVmScript in script) {
        return script as VmScriptLike as VmScript;
    }
    defineProperty(script, 'name', { value: VM_SCRIPT_NAME });
    defineProperty(script, kVmScript, { value: true });
    if (typeof source === 'string') {
        defineProperty(script, 'source', { value: source, configurable: true });
    } else if (source instanceof Uint8Array) {
        defineProperty(script, 'source', { value: '<buffer>', configurable: true });
    }
    return script as VmScript;
}

/** 生成 JS 函数 */
export function createScript(source: ScriptInput, code: string): VmScript {
    let script;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        script = new Function(...keys, code)(...values) as VmScriptLike;
        /* c8 ignore next 3 */
    } catch (error) {
        throw new Error(`Failed to create script`, { cause: error });
    }
    return wrapScript(source, script);
}
