import type { ScriptInput, TranspileOptions } from './types.js';
import { createScript, type VmScript } from './create-script.js';
import { generateBytecode } from './generate-bytecode.js';
import { compileWorker } from './worker-manager.js';
import { emitScript, CompileError } from './emit-script.js';
import { normalizeTranspileOptions, WORKER_MIN_LEN } from './options.js';

/**
 * 生成 MiraScript 对应的 JavaScript 代码
 */
export async function compileVerbose(
    this: void,
    source: ScriptInput,
    options?: TranspileOptions,
): Promise<{ script: VmScript; bytecode: Uint8Array<ArrayBuffer>; diagnostics: Uint32Array<ArrayBuffer> }> {
    options = normalizeTranspileOptions(options);
    if (source.length < WORKER_MIN_LEN) {
        const bc = await generateBytecode(source, options);
        const script = emitScript(source, bc, options);
        return {
            script,
            bytecode: bc[0]!, // Will throw if bytecode is undefined, but that should be handled by emitScript
            diagnostics: bc[1],
        };
    }
    const [bytecode, target, diagnostics] = await compileWorker(source, options);
    if (target == null) {
        throw new CompileError(source, diagnostics, options.fileName);
    }
    const script = createScript(source, options.input_mode ?? 'Script', target);
    return { script, bytecode, diagnostics };
}
