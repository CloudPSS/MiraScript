import type { VmScript } from '../vm/index.js';
import { keys, values } from '../vm/env.js';

const kVmScript = Symbol.for('mirascript.vm.script');
/** 生成 JS 函数 */
export function createScript(source: string, code: string): VmScript {
    let script;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        script = new Function(...keys, code)(...values) as VmScript;
    } catch (error) {
        throw new Error(`Failed to create script`, { cause: error });
    }
    Object.defineProperty(script, kVmScript, { value: true });
    Object.defineProperty(script, 'source', { value: source });
    return script;
}
