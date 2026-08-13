/// <reference lib="webworker" />
import type { Ready, Req, ResErr, ResOk } from './worker-core.js';

void Promise.resolve().then(async () => {
    const { compile } = await import('./worker-core.js');
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
});
