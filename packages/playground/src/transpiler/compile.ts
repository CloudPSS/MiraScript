import type { CompileOptions } from './options.js';

let loadModule: Promise<typeof import('mira-wasm')> | undefined;

/**
 * 生成 MiraScript 字节码
 */
export async function compile(code: string, _options: CompileOptions): Promise<[Uint8Array | undefined, Uint32Array]> {
    loadModule ??= import('mira-wasm');
    let compile_script;
    try {
        compile_script = (await loadModule).compile_script;
    } catch (error) {
        loadModule = undefined; // Reset on error to retry loading next time
        throw new Error(`Failed to load mira-wasm module`, { cause: error });
    }
    const chunk = compile_script(code);
    try {
        const diagnostics = chunk.diagnostics();
        const bytecode = chunk.chunk();
        if (bytecode == null) {
            return [undefined, diagnostics];
        }
        return [bytecode, diagnostics];
    } finally {
        chunk.free();
    }
}
