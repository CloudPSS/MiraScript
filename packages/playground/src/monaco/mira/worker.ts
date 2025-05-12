import * as wasm from 'mira-wasm';

/** 工作线程请求 */
export type Req<M extends keyof typeof wasm> = [number, M, ...Parameters<(typeof wasm)[M]>];
/** 工作线程请求 */
export type Res<M extends keyof typeof wasm> = [number, M, ReturnType<(typeof wasm)[M]>];

addEventListener('message', (ev) => {
    const data = ev.data as Req<keyof typeof wasm>;
    if (data.length < 2) return;
    const [id, method, ...args] = data;
    if (typeof id !== 'number' || typeof method !== 'string') return;
    if (!(method in wasm)) return;
    try {
        const result = wasm[method as 'keywords'](...(args as []));
        if (ArrayBuffer.isView(result)) {
            postMessage([id, method, result] as Res<typeof method>, { transfer: [result.buffer] });
        } else {
            postMessage([id, method, result] as Res<typeof method>);
        }
    } catch (ex) {
        postMessage([id, method, ex] as Res<typeof method>);
    }
});

setTimeout(() => postMessage('ready'));
