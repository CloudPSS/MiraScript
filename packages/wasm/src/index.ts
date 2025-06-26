import * as wasm from '../lib/wasm.js';
import type { Config, InputMode, DiagnosticPositionEncoding, ScriptInput } from './types.js';

export * from './types.js';
export { wasm };

export const ready = import('#loader').then(async ({ module }) =>
    wasm.default({
        module_or_path: await module,
    }),
);

/** 创建可重用的配置 */
export async function createConfig(config?: Config | wasm.Config): Promise<wasm.Config> {
    await ready;
    if (!config) return new wasm.Config();
    if (config instanceof wasm.Config) return config;
    const cfg = new wasm.Config();
    for (const key in config) {
        if (key === 'free') continue; // 忽略 free 方法
        if (!Object.hasOwn(config, key)) continue;
        let value = config[key as keyof Config] as never;
        if (key === 'input_mode') {
            value = wasm.InputMode[value as InputMode] satisfies wasm.InputMode as never;
        }
        if (key === 'diagnostic_position_encoding') {
            value = wasm.DiagnosticPositionEncoding[
                value as DiagnosticPositionEncoding
            ] satisfies wasm.DiagnosticPositionEncoding as never;
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
async function compileImpl<T>(
    compiler: (script: T, config: wasm.Config) => wasm.CompileResult,
    script: T,
    config: Config | wasm.Config,
): Promise<CompileResult> {
    const cfg = await createConfig(config);
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
export async function compile(script: ScriptInput, config: Config | wasm.Config): Promise<CompileResult> {
    return typeof script == 'string'
        ? compileImpl(wasm.compile, script, config)
        : compileImpl(wasm.compile_buffer, script, config);
}
