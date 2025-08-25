import { createConfig, type CompileResult, wasm } from '@mirascript/wasm';
import type { InputMode } from '@mirascript/mirascript';

/** Monaco 编译结果 */
export interface MonacoResult extends CompileResult {
    /** 格式化结果 */
    formatted?: string;
}

/** 请求参数 */
export type Req = [uri: string, version: number, script: string, mode: InputMode];
/** 编译结果 */
export type ResOk = [uri: string, version: number, result: MonacoResult];
/** 编译结果 */
export type ResErr = [uri: string, version: number, error: string];
/** 编译结果 */
export type Res = ResOk | ResErr;
/** Ready */
export type Ready = 'mirascript lsp ready';

const configTemplate = createConfig({
    diagnostic_position_encoding: 'Utf16',
    track_references: true,
    diagnostic_other: true,
    trivia: true,
    input_mode: 'Template',
});
const configScript = createConfig({
    diagnostic_position_encoding: 'Utf16',
    track_references: true,
    diagnostic_other: true,
    trivia: true,
    input_mode: 'Script',
});

/** 编译 */
export async function compile(script: string, mode: InputMode): Promise<MonacoResult> {
    const config = mode === 'Script' ? configScript : configTemplate;
    const compiler = new wasm.MonacoCompiler(script, await config);
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
        compiler.free();
    }
}

if (typeof Worker == 'function' && typeof addEventListener == 'function' && typeof postMessage == 'function') {
    // eslint-disable-next-line @typescript-eslint/no-misused-promises
    addEventListener('message', async (event: MessageEvent) => {
        const data = event.data as Req;
        if (!Array.isArray(data)) return;
        const [uri, version, script, mode] = data;
        try {
            const result = await compile(script, mode);
            const transfer = [];
            if (result.chunk) transfer.push(result.chunk.buffer);
            if (result.diagnostics) transfer.push(result.diagnostics.buffer);
            postMessage([uri, version, result] satisfies ResOk, { transfer });
        } catch (error) {
            postMessage([uri, version, (error as Error).message || String(error)] satisfies ResErr);
        }
    });
    postMessage('mirascript lsp ready' satisfies Ready);
}
