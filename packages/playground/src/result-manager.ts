import { type VmScript, compile, type InputMode } from '@mirascript/mirascript';
import { syntaxHighlight, print } from './utils.js';
import { getState } from './state-manager.js';
import type { ConsoleManager } from './console-manager.js';
import type { VmContext } from '@mirascript/mirascript';

/** 管理编译和运行结果 */
export function resultManager(
    consoleManager: ConsoleManager,
    elCompiledOutput: HTMLElement,
    elResultOutput: HTMLElement,
    globals: VmContext,
): () => Promise<void> {
    let fileCounter = 1;
    let cache: { fileName: string; mode: InputMode; source: string; script: VmScript } | null = null;
    /** 编译 */
    async function compileScript(): Promise<VmScript | undefined> {
        const { mode, source } = getState();
        const cacheHit = cache?.mode === mode && cache.source === source ? cache : null;
        const fileName =
            cacheHit?.fileName ??
            (mode === 'Script' ? `playground_${fileCounter++}.mira` : `playground_${fileCounter++}.miratpl`);

        const compStart = performance.now();
        try {
            const script = await compile(source, {
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
                    // eslint-disable-next-line @typescript-eslint/no-misused-promises
                    async () => {
                        if (elCompiledOutput.dataset['code'] !== compiledCode) return;
                        const highlightedJS = await syntaxHighlight(compiledCode, 'javascript');
                        if (elCompiledOutput.dataset['code'] !== compiledCode) return;
                        elCompiledOutput.innerHTML = /*html*/ `
                        <div class="section-title">Compiled JavaScript:</div>
                        <div class="compiled-code">${highlightedJS}</div>
                    `;
                    },
                    { timeout: 100 },
                );
            }
            return script;
        } catch (ex) {
            const compEnd = performance.now();
            const errorText = String(ex);
            elCompiledOutput.dataset['code'] = '';
            elCompiledOutput.innerHTML = /*html*/ `
            <div class="section-title">Compilation Error:</div>
            <div class="result-error">${errorText}</div>
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
    async function runScript(script: VmScript): Promise<void> {
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
        } catch (ex) {
            const execEnd = performance.now();
            const errorText = String(ex);
            elResultOutput.innerHTML = /* html */ `
            <div class="section-title">Execution Error:</div>
            <div class="result-error">${errorText}</div>
        `;
            consoleManager.error(`Execution failed in ${(execEnd - execStart).toFixed(3)}ms:`);
            consoleManager.error(ex as Error);
        }
    }

    return async () => {
        const script = await compileScript();
        if (!script) return;
        await runScript(script);
    };
}
