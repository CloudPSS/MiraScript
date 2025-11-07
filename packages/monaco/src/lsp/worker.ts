import { createConfig, type CompileResult, wasm, ready } from '@mirascript/wasm';
import type { InputMode } from '@mirascript/mirascript';
await ready;

/** Monaco 编译结果 */
export interface MonacoResult extends CompileResult {
    /** 格式化结果 */
    formatted?: string;
}

/** 缓存 Key (id, uri, inputMode) */
export type CacheKey = `${string}\0${string}\0${InputMode}`;
/** 请求参数 */
export type Req = [key: CacheKey, version: number, script: string, mode: InputMode];
/** 编译结果 */
export type ResOk = [key: CacheKey, version: number, result: MonacoResult];
/** 编译结果 */
export type ResErr = [key: CacheKey, version: number, error: Error];
/** 编译结果 */
export type Res = ResOk | ResErr;
/** Ready */
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

/** 编译 */
export function compile(script: string, mode: InputMode): MonacoResult {
    const config = mode === 'Script' ? configScript : configTemplate;
    const compiler = new wasm.MonacoCompiler(script, config);
    try {
        const parseOk = compiler.parse();
        if (!parseOk) {
            return { diagnostics: compiler.diagnostics(), chunk: undefined };
        }
        const chunk = compiler.emit();
        const formatted = compiler.format();
        return {
            diagnostics: compiler.diagnostics(),
            chunk,
            formatted,
        };
    } finally {
        try {
            compiler.free();
        } catch (ex) {
            /* 忽略错误 */
            // eslint-disable-next-line no-console
            console.error(ex);
        }
    }
}

if (typeof Worker == 'function' && typeof addEventListener == 'function' && typeof postMessage == 'function') {
    addEventListener('message', (event: MessageEvent) => {
        const data = event.data as Req;
        if (!Array.isArray(data)) return;
        const [key, version, script, mode] = data;
        try {
            const result = compile(script, mode);
            const transfer = [];
            if (result.chunk) transfer.push(result.chunk.buffer);
            if (result.diagnostics) transfer.push(result.diagnostics.buffer);
            postMessage([key, version, result] satisfies ResOk, { transfer });
        } catch (error) {
            const e = error instanceof Error ? error : new Error(String(error));
            postMessage([key, version, e] satisfies ResErr);
        }
    });
    postMessage('mirascript lsp ready' satisfies Ready);
}
