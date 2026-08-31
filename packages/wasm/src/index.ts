import type { Config, InputMode, DiagnosticPositionEncoding, ScriptInput } from '@mirascript/constants';
import * as wasm from '../lib/wasm.js';
import { loadModule } from '#loader';

export { wasm };

/** 初始化模块 */
export async function init(): Promise<void> {
    const module = await loadModule();
    await wasm.default({ module_or_path: module });
}

/** 创建可重用的配置 */
export function createConfig(config?: Config | wasm.CompileConfig): wasm.CompileConfig {
    if (!config) return new wasm.CompileConfig();
    if (config instanceof wasm.CompileConfig) return config;
    const cfg = new wasm.CompileConfig();
    for (const key in config) {
        if (key === 'free') continue; // 忽略 free 方法
        if (!Object.hasOwn(config, key)) continue;
        let value = config[key as keyof Config] as never;
        if (key === 'input_mode') {
            value = wasm.InputMode[value as InputMode] as never;
        } else if (key === 'diagnostic_position_encoding') {
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

/** 源码格式化选项 */
export interface FormatOptions {
    /** 制表符宽度。 */
    readonly tabSize?: number;
    /** 是否使用空格缩进。 */
    readonly insertSpaces?: boolean;
    /** 最大打印宽度。 */
    readonly printWidth?: number;
}

/** UTF-16 绝对偏移范围 */
export interface FormatRange {
    /** 起始 UTF-16 偏移。 */
    readonly start: number;
    /** 结束 UTF-16 偏移。 */
    readonly end: number;
}

/** 源码格式化编辑 */
export interface FormatEdit extends FormatRange {
    /** 替换文本。 */
    readonly text: string;
}

/** 源码格式化结果 */
export interface FormatResult {
    /** 编译诊断。 */
    readonly diagnostics: Uint32Array;
    /** 互不重叠的格式化编辑。 */
    readonly edits: readonly FormatEdit[];
}

/** 编译 */
function compileImpl<T>(
    compiler: (script: T, config: wasm.CompileConfig) => wasm.CompileResult,
    script: T,
    config: Config | wasm.CompileConfig,
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
export function compileSync(script: ScriptInput, config: Config | wasm.CompileConfig): CompileResult {
    return typeof script == 'string'
        ? compileImpl(wasm.compile, script, config)
        : compileImpl(wasm.compile_buffer, script, config);
}

/** 格式化 MiraScript 源码；ranges 使用 UTF-16 绝对偏移，省略时格式化整篇文档。 */
export function formatSync(
    source: string,
    config: Config | wasm.CompileConfig,
    options: FormatOptions = {},
    ranges?: readonly FormatRange[],
): FormatResult {
    const cfg = createConfig(config);
    const rawOptions = new wasm.FormatOptions();
    rawOptions.tab_size = options.tabSize ?? 2;
    rawOptions.insert_spaces = options.insertSpaces ?? true;
    rawOptions.print_width = options.printWidth ?? 80;
    const rawRanges = new Uint32Array((ranges?.length ?? 0) * 2);
    for (const [index, range] of (ranges ?? []).entries()) {
        rawRanges[index * 2] = range.start;
        rawRanges[index * 2 + 1] = range.end;
    }
    const result = wasm.format_sync(source, cfg, rawOptions, rawRanges);
    try {
        const encodedRanges = result.ranges();
        const edits: FormatEdit[] = [];
        for (let index = 0; index < result.edit_count(); index++) {
            edits.push({
                start: encodedRanges[index * 2]!,
                end: encodedRanges[index * 2 + 1]!,
                text: result.edit_text(index) ?? '',
            });
        }
        return { diagnostics: result.diagnostics(), edits };
    } finally {
        result.free();
        rawOptions.free();
        if (cfg !== config) cfg.free();
    }
}
