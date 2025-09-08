import { compile } from './compile.js';

addEventListener('message', (ev) => {
    const data = ev.data as [number, ...Parameters<typeof compile>];
    if (!Array.isArray(data)) return;
    const [seq, ...args] = data;
    if (typeof seq != 'number' || !args.length) return;
    void compile(...args)
        .then(([script, errors]) => {
            postMessage([seq, script, errors], { transfer: [errors.buffer] });
        })
        .catch((error) => {
            postMessage([seq, error instanceof Error ? error.message : String(error)]);
        });
});

compile('{}', {}).then(
    () => postMessage('ready'),
    (ex) => postMessage(ex),
);
