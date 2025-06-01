import { transpileCore } from './transpile.js';

addEventListener('message', (ev) => {
    const data = ev.data as [number, ...Parameters<typeof transpileCore>];
    if (!Array.isArray(data)) return;
    const [seq, code, options] = data;
    if (typeof seq != 'number') return;
    void transpileCore(code, options)
        .then(([script, errors]) => {
            postMessage([seq, script, errors], { transfer: [errors.buffer] });
        })
        .catch((error) => {
            postMessage([seq, undefined, error instanceof Error ? error.message : String(error)]);
        });
});

setTimeout(() => {
    postMessage('ready');
});
