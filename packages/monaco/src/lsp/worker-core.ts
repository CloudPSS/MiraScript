import type { InputMode } from '@mirascript/mirascript';
import type { BcModule } from '@mirascript/bindings/wasm';
import type { Tagged } from 'type-fest';

const { loadModule } = await import('@mirascript/bindings/wasm');
const module = await loadModule();
const { wasm, createConfig } = module;

/** Monaco 编译结果 */
export type MonacoResult = BcModule.CompileResult;
/** 缓存 Key (model id) */
export type CacheKey = Tagged<string, 'modelId'>;
/** Worker 请求标识。 */
export type RequestId = Tagged<number, 'requestId'>;

/** UTF-16 绝对偏移范围。 */
export interface OffsetRange {
    /** 起始偏移。 */
    readonly start: number;
    /** 结束偏移。 */
    readonly end: number;
}

/** Worker 格式化选项。 */
export interface WorkerFormatOptions {
    /** 制表符宽度。 */
    readonly tabSize: number;
    /** 是否使用空格缩进。 */
    readonly insertSpaces: boolean;
    /** 最大打印宽度。 */
    readonly printWidth: number;
}

/** Worker 请求公共字段。 */
interface BaseRequest {
    /** 请求标识。 */
    readonly id: RequestId;
    /** Monaco model 缓存键。 */
    readonly key: CacheKey;
    /** Model 版本。 */
    readonly version: number;
    /** 源码文本。 */
    readonly script: string;
    /** 编译输入模式。 */
    readonly mode: InputMode;
}

/** 编译请求。 */
export interface CompileRequest extends BaseRequest {
    /** 请求种类。 */
    readonly kind: 'compile';
}

/** 格式化请求。 */
export interface FormatRequest extends BaseRequest {
    /** 请求种类。 */
    readonly kind: 'format';
    /** 待格式化的 UTF-16 范围；省略时格式化整篇文档。 */
    readonly ranges?: readonly OffsetRange[];
    /** 格式化选项。 */
    readonly options: WorkerFormatOptions;
}

/** Worker 请求联合类型。 */
export type Req = CompileRequest | FormatRequest;
/** Worker 响应联合类型。 */
export type Res =
    | { readonly id: RequestId; readonly ok: true; readonly kind: 'compile'; readonly result: MonacoResult }
    | {
          readonly id: RequestId;
          readonly ok: true;
          readonly kind: 'format';
          readonly result: BcModule.FormatResult;
      }
    | { readonly id: RequestId; readonly ok: false; readonly error: string };
/** Worker 就绪消息。 */
export type Ready = 'mirascript lsp ready';

const configTemplate = createConfig({
    diagnostic_position_encoding: 'Utf16',
    diagnostic_tag: true,
    diagnostic_sourcemap: true,
    trivia: true,
    input_mode: 'Template',
});
const configScript = createConfig({
    diagnostic_position_encoding: 'Utf16',
    diagnostic_tag: true,
    diagnostic_sourcemap: true,
    trivia: true,
    input_mode: 'Script',
});

/** 选择与输入模式对应的可复用编译配置。 */
function config(mode: InputMode): BcModule.wasm.CompileConfig {
    return mode === 'Script' ? configScript : configTemplate;
}

/** 编译 */
export function compile(script: string, mode: InputMode): MonacoResult {
    const compiler = new wasm.MonacoCompiler(script, config(mode));
    try {
        const parseOk = compiler.parse();
        if (!parseOk) return { diagnostics: compiler.diagnostics(), chunk: undefined };
        const chunk = compiler.emit();
        return { diagnostics: compiler.diagnostics(), chunk };
    } finally {
        compiler.free();
    }
}

/** 按需格式化，不进入普通编译缓存。 */
export function formatSource(
    script: string,
    mode: InputMode,
    options: WorkerFormatOptions,
    ranges?: readonly OffsetRange[],
): BcModule.FormatResult {
    return module.formatSync(script, config(mode), options, ranges);
}
