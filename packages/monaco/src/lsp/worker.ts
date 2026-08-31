/// <reference lib="webworker" />
import type { Ready, Req, Res } from './worker-core.js';

void Promise.resolve().then(async () => {
    const { compile, formatSource } = await import('./worker-core.js');
    addEventListener('message', (event: MessageEvent<Req>) => {
        const request = event.data;
        if (!request || typeof request !== 'object' || typeof request.id !== 'number') return;
        try {
            if (request.kind === 'compile') {
                const result = compile(request.script, request.mode);
                const transfer: ArrayBuffer[] = [];
                if (result.chunk) transfer.push(result.chunk.buffer as ArrayBuffer);
                if (result.diagnostics) transfer.push(result.diagnostics.buffer as ArrayBuffer);
                postMessage({ id: request.id, ok: true, kind: 'compile', result } satisfies Res, { transfer });
            } else {
                const result = formatSource(request.script, request.mode, request.options, request.ranges);
                postMessage({ id: request.id, ok: true, kind: 'format', result } satisfies Res, {
                    transfer: [result.diagnostics.buffer as ArrayBuffer],
                });
            }
        } catch (error) {
            postMessage({
                id: request.id,
                ok: false,
                error: error instanceof Error ? error.message : String(error),
            } satisfies Res);
        }
    });
    postMessage('mirascript lsp ready' satisfies Ready);
});
