import type { CompileOptions, ScriptInput } from './types.js';

let module: Promise<typeof import('@mirascript/wasm') | typeof import('@mirascript/napi')> | undefined;

/** 加载模块 */
async function loadModule(): Promise<typeof import('@mirascript/wasm') | typeof import('@mirascript/napi')> {
    try {
        return await import('#compiler-bundle');
        /* c8 ignore next 5 */
    } catch (ex) {
        // eslint-disable-next-line no-console
        console.warn('Failed to load compiler bundle, falling back to @mirascript/wasm');
        return await import('@mirascript/wasm');
    }
}

/**
 * 生成 MiraScript 字节码
 */
export async function compileBytecode(
    script: ScriptInput,
    options: CompileOptions,
): Promise<[Uint8Array | undefined, Uint32Array]> {
    module ??= loadModule();
    let compile;
    try {
        ({ compile } = await module);
        /* c8 ignore next 4 */
    } catch (error) {
        module = undefined; // Reset on error to retry loading next time
        throw new Error(`Failed to load mira-wasm module`, { cause: error });
    }
    const result = await compile(script, options);
    return [result.chunk, result.diagnostics];
}
