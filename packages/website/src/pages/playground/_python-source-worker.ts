import type { PyodideInterface } from 'pyodide';
import type { PyProxy } from 'pyodide/ffi';
import type { PythonSourceRequest, PythonSourceResponse } from './_python-source-protocol';

/** Pyodide 静态资源清单。 */
type AssetsManifest = {
    wheels: {
        mirascript: string;
        typingExtensions: string;
    };
};

/** Python 字典代理。 */
type PythonNamespace = PyProxy & {
    /** 设置字典项。 */
    set(key: string, value: string): void;
};

/** 浏览器版 Pyodide loader。 */
type PyodideLoader = {
    /** 初始化 Pyodide。 */
    loadPyodide(options: { indexURL: string }): Promise<PyodideInterface>;
};

let pyodidePromise: Promise<PyodideInterface> | undefined;

/** 加载 Pyodide 与 MiraScript wheel。 */
async function initialize(assetsUrl: string): Promise<PyodideInterface> {
    pyodidePromise ??= (async () => {
        const baseUrl = new URL(assetsUrl, globalThis.location.href);
        const response = await fetch(new URL('manifest.json', baseUrl), { cache: 'no-cache' });
        if (!response.ok) throw new Error(`Failed to load Pyodide manifest: ${response.status} ${response.statusText}`);
        const manifest = (await response.json()) as AssetsManifest;
        const loaderUrl = new URL('pyodide.mjs', baseUrl).href;
        const loader = (await import(/* webpackIgnore: true */ loaderUrl)) as PyodideLoader;
        const pyodide = await loader.loadPyodide({ indexURL: baseUrl.href });
        await pyodide.loadPackage(new URL(manifest.wheels.typingExtensions, baseUrl).href);
        await pyodide.loadPackage(new URL(manifest.wheels.mirascript, baseUrl).href);
        return pyodide;
    })();
    try {
        return await pyodidePromise;
    } catch (error) {
        pyodidePromise = undefined;
        throw error;
    }
}

/** 生成 Python 源代码，但不调用编译得到的 script。 */
async function generate(request: PythonSourceRequest): Promise<string> {
    const pyodide = await initialize(request.assetsUrl);
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
