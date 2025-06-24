import { compile as c, createConfig, type CompileResult } from '@mirascript/wasm';
import type { InputMode } from 'mirascript';

/** 请求参数 */
export type Req = [uri: string, version: number, script: string, mode: InputMode];
/** 编译结果 */
export type ResOk = [uri: string, version: number, chunk: Uint8Array | undefined, diagnostics: Uint32Array];
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
export async function compile(script: string, mode: InputMode): Promise<CompileResult> {
    const config = mode === 'Script' ? configScript : configTemplate;
    return await c(script, await config);
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
            postMessage([uri, version, result.chunk, result.diagnostics] satisfies ResOk, { transfer });
        } catch (error) {
            postMessage([uri, version, (error as Error).message || String(error)] satisfies ResErr);
        }
    });
    postMessage('mirascript lsp ready' satisfies Ready);
}
