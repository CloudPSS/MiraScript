import type { Exports, Host } from './worker.js';
import type { editor, Uri } from '@private/monaco-editor';
import { Provider } from './providers/base.js';
import type { Monaco } from '../index.js';

const WORKERS = new Map<Monaco, editor.MonacoWebWorker<Exports>>();

/** 注册 worker */
export function registerWorker(monaco: Monaco): editor.MonacoWebWorker<Exports> {
    if (WORKERS.has(monaco)) {
        return WORKERS.get(monaco)!;
    }
    const registerWorker = monaco.utils?.registerWorker;
    if (typeof registerWorker == 'function') {
        registerWorker(
            'mirascript',
            () =>
                new Worker(new URL('./worker.js', import.meta.url), { name: '@mirascript/lsp-server', type: 'module' }),
        );
    }
    const { editor } = monaco;
    const monacoWorker = editor.createWebWorker<Exports>({
        label: 'mirascript',
        moduleId: '@mirascript/lsp-server',
        createData: {},
        host: {
            updateCompileResult: (uri, mode, version, diagnosticsBuffer, chunkBuffer) => {
                Provider.updateCompileResult(monaco, uri, mode, version, diagnosticsBuffer, chunkBuffer);
            },
        } satisfies Host,
    });
    WORKERS.set(monaco, monacoWorker);
    return monacoWorker;
}

/** 测试是否为 Uri */
function isUri(arg: unknown): arg is Uri {
    if (!arg || typeof arg != 'object') return false;
    return 'scheme' in arg && 'path' in arg;
}

/** 调用 worker 方法 */
export async function callWorker<const M extends keyof Exports>(
    monaco: Monaco,
    method: M,
    ...args: Parameters<Exports[M]>
): Promise<ReturnType<Exports[M]>> {
    const worker = registerWorker(monaco);
    const resources = args.filter((a: unknown) => isUri(a));
    const passArgs = args.map((arg: unknown) => (isUri(arg) ? arg.toString() : arg));
    const proxy = resources.length ? await worker.withSyncedResources(resources) : await worker.getProxy();
    return await (proxy[method as 'compileScript'](...(passArgs as [Uri])) as unknown as Promise<
        ReturnType<Exports[M]>
    >);
}
