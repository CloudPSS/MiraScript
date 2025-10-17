import type { CompileOptions, ScriptInput } from './types.ts';
import * as wasm from '@mirascript/wasm';

/** 字节码模块 */
type BcModule = typeof import('@mirascript/wasm') | typeof import('@mirascript/napi');

let module: BcModule;
let loading: Promise<void> | undefined;

/** 加载模块 */
export async function loadModule(): Promise<void> {
    if (module != null) return;
    if (loading) return loading;
    const p = (async () => {
        let mod: BcModule;
        try {
            mod = await import('#compiler-bundle');
            /* c8 ignore next 5 */
        } catch (ex) {
            // eslint-disable-next-line no-console
            console.warn('Failed to load compiler bundle, falling back to @mirascript/wasm');
            mod = wasm;
        }
        await wasm.ready;
        module = mod;
    })();
    void p.finally(() => {
        if (loading === p) {
            loading = undefined;
        }
    });
    loading = p;
    return p;
}

/** 检查模块加载情况 */
function checkModule(): void {
    if (module == null) {
        throw new Error('MiraScript compiler module is not loaded.');
    }
}

/**
 * 生成 MiraScript 字节码
 */
export function compileBytecodeSync(
    script: ScriptInput,
    options: CompileOptions,
): [Uint8Array | undefined, Uint32Array] {
    checkModule();
    const result = module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}

/**
 * 生成 MiraScript 字节码
 */
export async function compileBytecode(
    script: ScriptInput,
    options: CompileOptions,
): Promise<[Uint8Array | undefined, Uint32Array]> {
    await loadModule();
    const result = 'compile' in module ? await module.compile(script, options) : module.compileSync(script, options);
    return [result.chunk, result.diagnostics];
}
