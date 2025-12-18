import type { Config as WasmConfig, CompileResult as WasmCompileResult } from '../lib/wasm.js';
import type { Config, InputMode, DiagnosticPositionEncoding, ScriptInput } from './types.js';

export type { WasmCompileResult, WasmConfig };

export let wasm: typeof import('../lib/wasm.js');

const module = import('@mirascript/wasm/loader').then(async ({ module }) => module);

// 避免 vite 打包时出错
void import('../lib/wasm.js').then(async (mod) => {
    mod.initSync({ module: await module });
    wasm = mod;
});
export const ready = module.then(async () => {
    await new Promise<void>((resolve) => setTimeout(resolve, 1));
});

/** 创建可重用的配置 */
export function createConfig(config?: Config | WasmConfig): WasmConfig {
    if (!config) return new wasm.Config();
    if (config instanceof wasm.Config) return config;
    const cfg = new wasm.Config();
    for (const key in config) {
        if (key === 'free') continue; // 忽略 free 方法
        if (!Object.hasOwn(config, key)) continue;
        let value = config[key as keyof Config] as never;
        if (key === 'input_mode') {
            value = wasm.InputMode[value as InputMode] as never;
        }
        if (key === 'diagnostic_position_encoding') {
            value = wasm.DiagnosticPositionEncoding[value as DiagnosticPositionEncoding] as never;
        }
        if (value === undefined) continue;
        if (!(key in cfg)) continue;
        cfg[key as keyof Config] = value;
    }
    return cfg;
}

/** 编译结果 */
export interface CompileResult {
    /** 编译诊断 */
    readonly diagnostics: Uint32Array;
    /** 编译生成的字节码 */
    readonly chunk?: Uint8Array;
}

/** 编译 */
function compileImpl<T>(
    compiler: (script: T, config: WasmConfig) => WasmCompileResult,
    script: T,
    config: Config | WasmConfig,
): CompileResult {
    const cfg = createConfig(config);
    const result = compiler(script, cfg);
    try {
        const diagnostics = result.diagnostics();
        const chunk = result.chunk();
        return { diagnostics, chunk };
    } finally {
        result.free();
        // 只在 cfg 是新创建的情况下释放
        if (cfg !== config) {
            cfg.free();
        }
    }
}

/** 编译 MiraScript 代码 */
export function compileSync(script: ScriptInput, config: Config | WasmConfig): CompileResult {
    return typeof script == 'string'
        ? compileImpl(wasm.compile, script, config)
        : compileImpl(wasm.compile_buffer, script, config);
}

/** 获取诊断消息 */
export function getDiagnosticMessage(code: number): string | null {
    return wasm.get_diagnostic_message(code) || null;
}

/** 获取关键字列表 */
export function keywords(): string[] {
    return wasm.keywords();
}
