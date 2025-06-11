import type { CompileOptions, ParseOptions, ScriptInput } from './types.js';
import { toCompileFlags } from './utils.js';

let loadModule: Promise<typeof import('@mirascript/wasm')> | undefined;

const cachedTextEncoder = new TextEncoder();

/**
 * 生成 MiraScript 字节码
 */
export async function compileBytecode(
    script: ScriptInput,
    options: CompileOptions & ParseOptions,
): Promise<[Uint8Array, Uint8Array | undefined, Uint32Array]> {
    loadModule ??= import('@mirascript/wasm');
    let compileImpl;
    try {
        const { compileScript, compileTemplate } = await loadModule;
        compileImpl = options.mode === 'template' ? compileTemplate : compileScript;
    } catch (error) {
        loadModule = undefined; // Reset on error to retry loading next time
        throw new Error(`Failed to load mira-wasm module`, { cause: error });
    }
    const compileOptions = toCompileFlags({ ...options, HideDiagnosticOther: options.HideDiagnosticOther ?? true });
    const scriptBuf = ArrayBuffer.isView(script) ? script : cachedTextEncoder.encode(script);
    const result = compileImpl(scriptBuf, compileOptions);
    return [scriptBuf, result.chunk, result.diagnostics];
}
