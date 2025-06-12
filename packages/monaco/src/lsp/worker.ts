import { compileScript, compileTemplate } from '@mirascript/wasm';
import type { ParseMode } from 'mirascript';
import { toCompileFlags } from 'mirascript/subtle';

/** 请求参数 */
export type Req = [uri: string, version: number, script: string, mode: ParseMode];
/** 编译结果 */
export type ResOk = [uri: string, version: number, chunk: Uint8Array | undefined, diagnostics: Uint32Array];
/** 编译结果 */
export type ResErr = [uri: string, version: number, error: string];
/** 编译结果 */
export type Res = ResOk | ResErr;
/** Ready */
export type Ready = 'mirascript lsp ready';

const flags = toCompileFlags({
    UseUtf16: true,
    TrackReferences: true,
    HideDiagnosticOther: false,
});
const encoder = new TextEncoder();

addEventListener('message', (event: MessageEvent) => {
    const data = event.data as Req;
    if (!Array.isArray(data)) return;
    const [uri, version, script, mode] = data;
    try {
        const compiler = mode === 'template' ? compileTemplate : compileScript;
        const result = compiler(encoder.encode(script), flags);
        postMessage([uri, version, result.chunk, result.diagnostics] satisfies ResOk);
    } catch (error) {
        postMessage([uri, version, (error as Error).message || String(error)] satisfies ResErr);
    }
});
postMessage('mirascript lsp ready' satisfies Ready);
