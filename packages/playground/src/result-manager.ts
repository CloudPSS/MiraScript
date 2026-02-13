import type { VmScript, InputMode, VmAny, VmContext } from '@mirascript/mirascript';
import type { ConsoleManager } from './console-manager.js';
import { print, escapeHtml } from './utils.js';
import { getState } from './state-manager.js';
import { mirascript, monaco, ready } from './loader.js';
import { createEditor } from './monaco.js';
import operationDts from './operation.d.ts?raw';

/** 管理编译和运行结果 */
export function resultManager(
    consoleManager: ConsoleManager,
    elCompiledOutput: HTMLElement,
    elResultOutput: HTMLElement,
    globals: VmContext,
): () => Promise<VmAny> {
    let cache: { fileName: string; mode: InputMode; source: string; script: VmScript } | null = null;
    let editor: import('@private/monaco-editor').editor.IStandaloneCodeEditor | null = null;
    /** 编译 */
    async function compileScript(): Promise<VmScript | undefined> {
        await ready;
        const { mode, source } = getState();
        const cacheHit = cache?.mode === mode && cache.source === source ? cache : null;
        const fileName = cacheHit?.fileName ?? (mode === 'Script' ? `playground.mira` : `playground.miratpl`);

        const compStart = performance.now();
        try {
            const script = await mirascript.compile(source, {
                pretty: true,
                sourceMap: true,
                input_mode: mode,
                fileName,
            });
            const compEnd = performance.now();
            consoleManager.info(`Compilation completed successfully in ${(compEnd - compStart).toFixed(3)}ms`);

            if (cacheHit) {
                // 重新编译仅用于计时
                return cacheHit.script;
            }

            cache = { fileName, mode, source, script };
            // 显示编译结果
            const compiledCode = script.toString();
            if (elCompiledOutput.dataset['code'] !== compiledCode) {
                elCompiledOutput.dataset['code'] = compiledCode;

                requestIdleCallback(
                    () => {
                        if (elCompiledOutput.dataset['code'] !== compiledCode) return;
                        if (!editor) {
                            monaco.typescript.javascriptDefaults.addExtraLib(operationDts, `file:///operation.d.ts`);
                            const div = document.createElement('div');
                            div.id = 'compiled-editor';
                            elCompiledOutput.replaceChildren(div);
                            editor = createEditor(div, {
                                readOnly: true,
                                minimap: { enabled: false },
                                wordWrap: 'on',
                                wrappingIndent: 'deepIndent',
                                language: 'javascript',
                                value: compiledCode,
                            });
                        } else {
                            editor.setValue(compiledCode);
                            elCompiledOutput.replaceChildren(editor.getContainerDomNode());
                            editor.layout();
                        }
                    },
                    { timeout: 100 },
                );
            }
            return script;
        } catch (ex) {
            cache = null;
            const compEnd = performance.now();
            const errorText = String(ex);
            elCompiledOutput.dataset['code'] = '';
            elCompiledOutput.innerHTML = /*html*/ `
            <div class="section-title">Compilation Error:</div>
            <div class="result-error">${escapeHtml(errorText)}</div>
        `;
            elResultOutput.innerHTML = /*html*/ `
            <div class="section-title">Execution Result:</div>
            <div class="result-error">Compilation failed</div>
        `;
            consoleManager.error(`Compilation failed in ${(compEnd - compStart).toFixed(3)}ms:`);
            consoleManager.error(ex as Error);
            return undefined;
        }
    }

    /** 运行 */
    async function runScript(script: VmScript): Promise<VmAny> {
        const execStart = performance.now();
        try {
            const execResult = script(globals);
            const execEnd = performance.now();
            if (typeof execResult == 'string' && /^\s*<!\s*doctype\s+html\s*>/iu.test(execResult)) {
                elResultOutput.innerHTML = /* html */ `
                <div class="section-title">Execution Result:</div>
                <iframe class="result-success html" srcdoc="${execResult.replaceAll('"', '&quot;')}"></iframe>
            `;
            } else {
                const resultText = await print(execResult);
                elResultOutput.innerHTML = /* html */ `
                <div class="section-title">Execution Result:</div>
                <div class="result-success">${resultText}</div>
            `;
            }
            consoleManager.info(`Execution completed successfully in ${(execEnd - execStart).toFixed(3)}ms`);
            return execResult;
        } catch (ex) {
            const execEnd = performance.now();
            const errorText = String(ex);
            elResultOutput.innerHTML = /* html */ `
            <div class="section-title">Execution Error:</div>
            <div class="result-error">${escapeHtml(errorText)}</div>
        `;
            consoleManager.error(`Execution failed in ${(execEnd - execStart).toFixed(3)}ms:`);
            consoleManager.error(ex as Error);
            return undefined;
        }
    }

    return async () => {
        const script = await compileScript();
        if (!script) return;
        return await runScript(script);
    };
}
