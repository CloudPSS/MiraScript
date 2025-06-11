import type { VmScript } from '../vm/index.js';
import { keys, values } from '../vm/env.js';
import type { ScriptInput } from './types.js';

const kVmScript = Symbol.for('mirascript.vm.script');
/** 生成 JS 函数 */
export function createScript(source: ScriptInput, code: string): VmScript {
    let script;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        script = new Function(...keys, code)(...values) as VmScript;
    } catch (error) {
        throw new Error(`Failed to create script`, { cause: error });
    }
    Object.defineProperty(script, kVmScript, { value: true });
    if (typeof source === 'string') {
        Object.defineProperty(script, 'source', { value: source });
    } else if (source instanceof Uint8Array) {
        Object.defineProperty(script, 'source', { value: '<buffer>' });
    }
    return script;
}
