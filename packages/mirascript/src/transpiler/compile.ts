import { toCompileFlags, type CompileOptions } from './options.js';

let loadModule: Promise<typeof import('@mirascript/wasm')> | undefined;

const textEncoder = new TextEncoder();

/**
 * 生成 MiraScript 字节码
 */
export async function compile(
    code: string,
    options: CompileOptions,
): Promise<[Uint8Array, Uint8Array | undefined, Uint32Array]> {
    loadModule ??= import('@mirascript/wasm');
    let compile_script, CompileFlag;
    try {
        ({ compile_script, CompileFlag } = await loadModule);
    } catch (error) {
        loadModule = undefined; // Reset on error to retry loading next time
        throw new Error(`Failed to load mira-wasm module`, { cause: error });
    }
    const compileOptions = { ...options, HideDiagnosticOther: options.HideDiagnosticOther ?? true };
    const flags = toCompileFlags(compileOptions, CompileFlag);
    const codeBuffer = textEncoder.encode(code);
    const chunk = compile_script(codeBuffer, flags);
    try {
        const diagnostics = chunk.diagnostics();
        const bytecode = chunk.chunk();
        if (bytecode == null) {
            return [codeBuffer, undefined, diagnostics];
        }
        return [codeBuffer, bytecode, diagnostics];
    } finally {
        chunk.free();
    }
}
