/// <reference lib="webworker" />
import type pyodide from 'pyodide';
import type { PyProxy } from 'pyodide/ffi';
import type { PythonSourceRequest, PythonSourceResponse } from './_python-source-protocol.js';
import { devDependencies } from '../../../package.json';

/** Python 字典代理。 */
type PythonNamespace = PyProxy & {
    /** 设置字典项。 */
    set(key: string, value: string): void;
};

/** 加载 Pyodide loader */
async function loadPyodideLoader(): Promise<(typeof pyodide)['loadPyodide']> {
    const PYODIDE_VERSION = devDependencies.pyodide.replaceAll(/^[~^]/g, '');
    const PYODIDE_CDN_URL = `https://fastly.jsdelivr.net/pyodide/v${PYODIDE_VERSION}/full/`;

    // 在 worker 中禁用 importScripts，防止 Pyodide loader 使用 importScripts 判断环境为 classic worker
    const backup = globalThis.importScripts;
    globalThis.importScripts = () => {
        throw new Error('importScripts is disabled in this worker');
    };
    try {
        const loaderUrl = new URL('pyodide.mjs', PYODIDE_CDN_URL).href;
        const loader = (await import(/* webpackIgnore: true */ loaderUrl)) as typeof pyodide;
        return loader.loadPyodide;
    } finally {
        globalThis.importScripts = backup;
    }
}

/** 加载 Pyodide Wheels */
async function loadWheels(): Promise<
    Array<readonly [name: string, url: string, data: Promise<Uint8Array<ArrayBuffer>>]>
> {
    const files = import.meta.webpackContext('../../../pyodide.g.assets/', {
        regExp: /\.(whl)$/,
        mode: 'sync',
        recursive: false,
    });
    const entries = await Promise.all(
        files.keys().map(async (k) => {
            const name = k.replace(/^\.\//, '');
            const url = (await files(k)) as string;
            const data = fetch(url).then(async (res) => new Uint8Array(await res.arrayBuffer()));
            return [name, url, data] as const;
        }),
    );
    return entries;
}

/** 加载 Pyodide */
async function loadPyodide(): Promise<pyodide.PyodideInterface> {
    const wheels = await loadWheels();
    const loader = await loadPyodideLoader();
    const pyodide = await loader({ packages: ['micropip'] });
    await Promise.all(
        wheels.map(async ([name, _, data]) => {
            await (pyodide.FS as typeof import('node:fs/promises')).writeFile(name, await data);
        }),
    );
    const micropip = pyodide.pyimport('micropip') as { install: (packages: string[]) => Promise<void> };
    await micropip.install(wheels.map(([name]) => `emfs:${name}`));
    return pyodide;
}

let pyodidePromise: Promise<pyodide.PyodideInterface> | undefined;
/** 加载 Pyodide 与 MiraScript wheel。 */
async function initialize(): Promise<pyodide.PyodideInterface> {
    pyodidePromise ??= loadPyodide();
    try {
        return await pyodidePromise;
    } catch (error) {
        pyodidePromise = undefined;
        throw error;
    }
}

/** 生成 Python 源代码，但不调用编译得到的 script。 */
async function generate(request: PythonSourceRequest): Promise<string> {
    const pyodide = await initialize();
    const namespace = pyodide.runPython('dict()') as unknown as PythonNamespace;
    try {
        namespace.set('source', request.source);
        namespace.set('input_mode', request.mode.toLowerCase());
        namespace.set('filename', request.fileName);
        const result = pyodide.runPython(
            String.raw`
import ast
from mirascript import compile

script, diagnostics = compile(source, input_mode=input_mode, filename=filename)
if script is None:
    raise RuntimeError("Compilation failed:\n" + "\n".join(map(str, diagnostics)))
if script.ast is None:
    raise RuntimeError("Compiled script does not contain a Python AST")
ast.unparse(script.ast)
`,
            { globals: namespace },
        ) as unknown;
        if (typeof result !== 'string') throw new TypeError('Python source generator returned a non-string value.');
        return result;
    } finally {
        namespace.destroy();
    }
}

const workerGlobal = globalThis as unknown as {
    /** 监听 worker 消息。 */
    addEventListener(type: 'message', listener: (event: MessageEvent<PythonSourceRequest>) => void): void;
    /** 发送 worker 消息。 */
    postMessage(message: PythonSourceResponse): void;
};

workerGlobal.addEventListener('message', (event) => {
    const request = event.data;
    void generate(request).then(
        (source) => workerGlobal.postMessage({ id: request.id, source }),
        (error: unknown) =>
            workerGlobal.postMessage({ id: request.id, error: error instanceof Error ? error.message : String(error) }),
    );
});
