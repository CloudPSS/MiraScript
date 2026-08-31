import type { InputMode } from '@mirascript/constants';
import { editor } from '../monaco-api.js';
import type {
    CacheKey,
    CompileRequest,
    FormatRequest,
    OffsetRange,
    Ready,
    Req,
    Res,
    RequestId,
    WorkerFormatOptions,
} from './worker-core.js';
import { CompileResult } from './compile-result.js';
import { makeModelMarkers } from './diagnostics.js';

/** 编译缓存项。 */
type CacheValue = {
    readonly version: number;
    readonly result: Promise<CompileResult>;
    readonly mode: InputMode;
    lastAccess: number;
};

const cache = new Map<CacheKey, CacheValue>();
let worker: Promise<Worker> | undefined;
let requestSequence = 0;

const CACHE_MAX_AGE = 30000;
setInterval(() => {
    const now = Date.now();
    for (const [key, { lastAccess }] of cache) {
        if (now - lastAccess > CACHE_MAX_AGE) cache.delete(key);
    }
}, CACHE_MAX_AGE);

/** 生成当前进程内唯一的请求标识。 */
function nextRequestId(): RequestId {
    requestSequence = (requestSequence + 1) % Number.MAX_SAFE_INTEGER;
    return requestSequence as RequestId;
}

/** 从 Monaco model 推断编译输入模式。 */
function modeOf(model: editor.ITextModel): InputMode {
    return model.getLanguageId() === 'mirascript-template' ? 'Template' : 'Script';
}

/** 创建并等待 LSP worker 就绪。 */
async function getWorker(): Promise<Worker> {
    if (!worker) {
        const instance = new Worker(new URL('#lsp/worker', import.meta.url), {
            type: 'module',
            name: '@mirascript/lsp-server',
        });
        worker = new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                cleanUp();
                reject(new Error('Worker did not respond in time'));
            }, 30000);
            const cleanUp = () => {
                clearTimeout(timeout);
                instance.removeEventListener('error', onError);
                instance.removeEventListener('message', onMessage);
            };
            const onError = (event: ErrorEvent) => {
                cleanUp();
                reject(new Error(`Worker failed to start: ${event.message}`));
            };
            const onMessage = (event: MessageEvent<Ready>) => {
                if (event.data !== 'mirascript lsp ready') return;
                cleanUp();
                resolve(instance);
            };
            instance.addEventListener('error', onError);
            instance.addEventListener('message', onMessage);
        });
    }
    return worker;
}

/** 发送带标识的 worker 请求并等待对应响应。 */
async function requestWorker(request: Req): Promise<Res> {
    const instance = await getWorker();
    return new Promise<Res>((resolve) => {
        const onMessage = (event: MessageEvent<Res>) => {
            if (event.data?.id !== request.id) return;
            instance.removeEventListener('message', onMessage);
            resolve(event.data);
        };
        instance.addEventListener('message', onMessage);
        instance.postMessage(request);
    });
}

let compileImpl: typeof import('./worker-core.js').compile;
/** 在当前线程执行编译。 */
async function compileSync(request: CompileRequest): Promise<CompileResult> {
    compileImpl ??= (await import('./worker-core.js')).compile;
    const result = compileImpl(request.script, request.mode);
    return new CompileResult(request.key, request.version, request.script, result);
}

/** 优先使用 worker 编译，失败时回退当前线程。 */
async function compileWorker(request: CompileRequest): Promise<CompileResult> {
    try {
        const response = await requestWorker(request);
        if (!response.ok) throw new Error(response.error);
        if (response.kind !== 'compile') throw new Error('Unexpected formatter response to compile request');
        return new CompileResult(request.key, request.version, request.script, response.result);
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('Failed to use MiraScript worker, falling back to the current thread:', error);
        USE_WORKER = false;
        worker = undefined;
        return compileSync(request);
    }
}

let USE_WORKER = typeof Worker === 'function';

/** 编译并设置缓存 */
export async function compile(model: editor.ITextModel): Promise<CompileResult> {
    const version = model.getVersionId();
    const mode = modeOf(model);
    const key = model.id as CacheKey;
    const cached = cache.get(key);
    if (cached?.version === version && cached.mode === mode) {
        cached.lastAccess = Date.now();
        return cached.result;
    }

    const script = model.getValue();
    const request: CompileRequest = { kind: 'compile', id: nextRequestId(), key, version, script, mode };
    const result = USE_WORKER ? compileWorker(request) : compileSync(request);
    void result.then(async (compiled) => {
        if (model.isDisposed()) return;
        const setModelMarkers = editor?.setModelMarkers;
        if (typeof setModelMarkers !== 'function') return;
        const markers = await makeModelMarkers(model, compiled);
        if (markers) setModelMarkers(model, 'mirascript', markers);
    });
    const item: CacheValue = { version, lastAccess: Date.now(), mode, result };
    cache.set(key, item);
    result.catch(() => {
        if (cache.get(key) === item) cache.delete(key);
    });
    return result;
}

let formatImpl: typeof import('./worker-core.js').formatSource;

/** 对指定 model 发起独立的按需格式化请求。 */
export async function formatModel(
    model: editor.ITextModel,
    ranges: readonly OffsetRange[] | undefined,
    options: WorkerFormatOptions,
): Promise<{
    readonly diagnostics: Uint32Array;
    readonly edits: ReadonlyArray<{ start: number; end: number; text: string }>;
}> {
    const request: FormatRequest = {
        kind: 'format',
        id: nextRequestId(),
        key: model.id as CacheKey,
        version: model.getVersionId(),
        script: model.getValue(),
        mode: modeOf(model),
        ranges,
        options,
    };
    if (!USE_WORKER) {
        formatImpl ??= (await import('./worker-core.js')).formatSource;
        return formatImpl(request.script, request.mode, request.options, request.ranges);
    }
    try {
        const response = await requestWorker(request);
        if (!response.ok) throw new Error(response.error);
        if (response.kind !== 'format') throw new Error('Unexpected compile response to format request');
        return response.result;
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('Failed to use MiraScript worker, formatting on the current thread:', error);
        USE_WORKER = false;
        worker = undefined;
        formatImpl ??= (await import('./worker-core.js')).formatSource;
        return formatImpl(request.script, request.mode, request.options, request.ranges);
    }
}
