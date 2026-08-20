import type { ScriptInput, TranspileOptions } from './types.js';
import { createScript, type VmScript } from './create-script.js';
import { emitIL } from './emit-il.js';
import { emitScript, CompileError } from './emit-script.js';
import { generateBytecode } from './generate-bytecode.js';
import { DiagnosticCode, parseDiagnostics } from './diagnostic.js';
import { compileILWorker } from './worker-manager.js';
import { normalizeTranspileOptions, WORKER_MIN_LEN } from './options.js';

/** 同一次编译产生的可执行脚本与 MiraScript IL。 */
export interface VmILCompileResult {
    /** 可执行脚本。 */
    readonly script: VmScript;
    /** 可读 MiraScript IL。 */
    readonly il: string;
}

/** 编译 MiraScript，并返回可执行脚本和对应 IL。 */
export async function compileWithIL(
    this: void,
    source: ScriptInput,
    options?: TranspileOptions,
): Promise<VmILCompileResult> {
    options = normalizeTranspileOptions(options);
    options.diagnostic_sourcemap = true;
    if (!options.diagnostic_position_encoding) {
        options.diagnostic_position_encoding = 'Utf16';
    }
    if (source.length < WORKER_MIN_LEN) {
        const bytecode = await generateBytecode(source, options);
        const script = emitScript(source, bytecode, options);
        const { sourcemaps } = parseDiagnostics(source, bytecode[1], (code) => code === DiagnosticCode.SourceMap);
        return {
            script,
            il: emitIL(bytecode[0]!, {
                source,
                ranges: sourcemaps,
            }),
        };
    }
    const [target, il, diagnostics] = await compileILWorker(source, options);
    if (target == null || il == null) {
        throw new CompileError(source, diagnostics, options.fileName);
    }
    return {
        script: createScript(source, options.input_mode ?? 'Script', target),
        il,
    };
}
