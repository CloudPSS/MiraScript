import type { CompileOptions, ScriptInput } from './types.js';

let loadModule: Promise<typeof import('@mirascript/wasm')> | undefined;

/**
 * 生成 MiraScript 字节码
 */
export async function compileBytecode(
    script: ScriptInput,
    options: CompileOptions,
): Promise<[Uint8Array | undefined, Uint32Array]> {
    loadModule ??= import('@mirascript/wasm');
    let compile;
    try {
        ({ compile } = await loadModule);
    } catch (error) {
        loadModule = undefined; // Reset on error to retry loading next time
        throw new Error(`Failed to load mira-wasm module`, { cause: error });
    }
    const result = compile(script, options);
    return [result.chunk, result.diagnostics];
}
