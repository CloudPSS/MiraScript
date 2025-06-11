import { compile } from './compile.js';

addEventListener('message', (ev) => {
    const data = ev.data as [number, ...Parameters<typeof compile>];
    if (!Array.isArray(data)) return;
    const [seq, ...args] = data;
    if (typeof seq != 'number') return;
    void compile(...args)
        .then(([buf, script, errors]) => {
            postMessage([seq, buf, script, errors], { transfer: [buf.buffer, errors.buffer] });
        })
        .catch((error) => {
            postMessage([seq, undefined, error instanceof Error ? error.message : String(error)]);
        });
});

setTimeout(() => {
    postMessage('ready');
});
