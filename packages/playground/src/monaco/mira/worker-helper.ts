import type * as worker from './worker.js';
import { utils, editor, Uri } from '@private/monaco-editor';

utils.registerWorker(
    'mirascript',
    () => new Worker(new URL('./worker.js', import.meta.url), { name: '@mirascript/lsp-server', type: 'module' }),
);

/** 编译结果 */
export interface CompileResult {
    /** 错误信息 */
    errors: Uint32Array;
    /** 代码信息 */
    chunk?: Uint8Array;
}
const compileResult = new Map<string, CompileResult>();
const monacoWorker = editor.createWebWorker({
    label: 'mirascript',
    moduleId: '@mirascript/lsp-server',
    createData: {},
    host: {
        updateCompileResult(uri, errors, chunk) {
            compileResult.set(uri, {
                errors: new Uint32Array(errors),
                chunk: chunk ? new Uint8Array(chunk) : undefined,
            });
        },
    } satisfies worker.Host,
});

/** 获取编译结果 */
export function getCompileResult(uri: Uri): CompileResult | undefined {
    return compileResult.get(uri.toString());
}

/** 调用 worker 方法 */
export async function callWorker<const M extends keyof typeof worker>(
    method: M,
    ...args: Parameters<(typeof worker)[M]>
): Promise<ReturnType<(typeof worker)[M]>> {
    const resources = args.filter((a: unknown) => a instanceof Uri);
    const passArgs = args.map((arg: unknown) => (arg instanceof Uri ? arg.toString() : arg));
    const proxy = resources.length ? await monacoWorker.withSyncedResources(resources) : await monacoWorker.getProxy();
    return await ((proxy as typeof worker)[method as 'keywords'](...(passArgs as [])) as unknown as Promise<
        ReturnType<(typeof worker)[M]>
    >);
}
