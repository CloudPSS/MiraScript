import './load-module.js';
import type { ScriptInput, TranspileOptions } from './types.js';
import type { VmScript } from './create-script.js';
import { compileFast } from './compile-fast.js';
import { generateBytecodeSync } from './generate-bytecode.js';
import { emitScript, CompileError } from './emit-script.js';
import { normalizeTranspileOptions } from './options.js';
import { compileVerbose } from './verbose.js';

export * from './types.js';
export type { VmScript };
export { CompileError };

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compile(this: void, source: ScriptInput, options?: TranspileOptions): Promise<VmScript> {
    if (typeof source == 'string') {
        options = normalizeTranspileOptions(options);
        const result = compileFast(source, options);
        if (result) return result;
    }
    const { script } = await compileVerbose(source, options);
    return script;
}

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export function compileSync(this: void, source: ScriptInput, options?: TranspileOptions): VmScript {
    options = normalizeTranspileOptions(options);
    if (typeof source == 'string') {
        const result = compileFast(source, options);
        if (result) return result;
    }
    const bc = generateBytecodeSync(source, options);
    return emitScript(source, bc, options);
}
