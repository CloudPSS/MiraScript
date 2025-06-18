import * as wasm from '../lib/wasm.js';

export { DiagnosticCode, OpCode, DiagnosticPositionEncoding } from '../lib/wasm.js';
export { wasm };
/**
 * 配置选项
 */
export type Config = Partial<Omit<wasm.Config, 'free' | 'input_mode'>> & {
    input_mode?: InputMode;
};
/** 输入模式 */
export type InputMode = keyof typeof wasm.InputMode;
/** 编译输入，支持字符串和 UTF-8 字节数组 */
export type ScriptInput = string | Uint8Array;

/** 创建可重用的配置 */
export function createConfig(config?: Config | wasm.Config): wasm.Config {
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
    readonly chunk: Uint8Array | undefined;
}

/** 编译 */
function compileImpl<T>(
    compiler: (script: T, config: wasm.Config) => wasm.CompileResult,
    script: T,
    config: Config | wasm.Config,
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
export function compile(script: string | Uint8Array, config: Config | wasm.Config): CompileResult {
    return typeof script == 'string'
        ? compileImpl(wasm.compile, script, config)
        : compileImpl(wasm.compile_buffer, script, config);
}
