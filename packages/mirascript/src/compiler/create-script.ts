import { keys, values } from '../vm/env.js';
import type { VmScript, VmScriptLike } from '../vm/types/script.js';
import type { ScriptInput } from './types.js';

const kVmScript = Symbol.for('mirascript.vm.script');

/** 生成 JS 函数 */
export function wrapScript(source: ScriptInput, script: VmScriptLike): VmScript {
    if (kVmScript in script) {
        return script as VmScriptLike as VmScript;
    }
    Object.defineProperty(script, kVmScript, { value: true });
    if (typeof source === 'string') {
        Object.defineProperty(script, 'source', { value: source });
    } else if (source instanceof Uint8Array) {
        Object.defineProperty(script, 'source', { value: '<buffer>' });
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
