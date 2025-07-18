import './index.css';
import * as monaco from '@private/monaco-editor';
import { KeyCode, KeyMod } from '@private/monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';
import {
    type VmAny,
    type VmScript,
    VmExtern,
    createVmContext,
    compile,
    VmModule,
    VmFunction,
    type InputMode,
} from 'mirascript';
import { ConsoleManager } from './console-manager.js';
import { EXAMPLES } from './examples.js';
import { syntaxHighlight, print } from './utils.js';
import { getState, setState, type ThemeMode } from './state-manager.js';

const arr = [1, 2, [1, 2], { x: 0 }];
arr[100] = 100; // make a sparse array

// 初始化控制台管理器
const consoleManager = new ConsoleManager(document.querySelector<HTMLDivElement>('#console-output')!);

/** 创建简单的 debug_print 函数 */
function debugPrint(...args: VmAny[]) {
    const messages = args.map(async (arg) => {
        if (typeof arg === 'string') return arg;
        return print(arg);
    });
    consoleManager.log(Promise.all(messages).then((message) => message.join(' ')));
}

const globals = createVmContext(
    {
        extern_arr: new VmExtern(arr),
        obj: { a: [], b: 1, c: '2', d: { e: 3 } },
        arr: [1, 2, 3],
        long_str: 'Long string content'.repeat(10000),
        mod: new VmModule('test', {
            sin: createVmContext().sin,
            inner: new VmModule('inner', {}),
        }),
        debug_print: VmFunction(debugPrint, {
            fullName: 'debug_print',
            summary: 'Print debug messages to console',
        }),
        name: 'MiraScript', // for template examples
    },
    {
        extern_obj: { a: [], b: 1, c: '2', d: { e: 3 }, sin: createVmContext().sin },
        globalThis,
    },
);

registerMiraScript(monaco, () => globals);

let modelIndex = 1;
const createModel = () => {
    const { source, mode } = getState();
    // 使用唯一的URI避免冲突
    const uri = monaco.Uri.parse(`file:///playground-${modelIndex++}.${mode === 'Template' ? 'miratpl' : 'mira'}`);
    const model = monaco.editor.createModel(source, mode === 'Template' ? 'mirascript-template' : 'mirascript', uri);
    model.setEOL(monaco.editor.EndOfLineSequence.LF);
    return model;
};

const updateModel = () => {
    const oldModel = editor.getModel();
    const { source, mode } = getState();
    if (oldModel?.getLanguageId() === (mode === 'Template' ? 'mirascript-template' : 'mirascript')) {
        // 如果语言模式没有变化，直接更新内容
        oldModel.setValue(source);
        return;
    }
    const newModel = createModel();
    editor.setModel(newModel);
    oldModel?.dispose();
};

// UI 元素
const elEditor = document.querySelector<HTMLDivElement>('#editor')!;
const elThemeSelect = document.querySelector<HTMLSelectElement>('#theme-select')!;
const elExampleSelect = document.querySelector<HTMLSelectElement>('#example-select')!;
const elModeSelect = document.querySelector<HTMLSelectElement>('#mode-select')!;
const elRunBtn = document.querySelector<HTMLButtonElement>('#run-btn')!;
const elCompiledOutput = document.querySelector<HTMLDivElement>('#compiled-output')!;
const elResultOutput = document.querySelector<HTMLDivElement>('#result-output')!;

/** 初始化主题选择器 */
function initThemeSelector() {
    const { theme } = getState();
    elThemeSelect.value = theme;

    // 应用初始主题
    applyTheme(theme);

    elThemeSelect.addEventListener('change', () => {
        const newTheme = elThemeSelect.value as ThemeMode;
        setState({ theme: newTheme });
        applyTheme(newTheme);
    });
}

const systemTheme = matchMedia('(prefers-color-scheme: dark)');
/** 应用主题 */
function applyTheme(theme: ThemeMode) {
    document.documentElement.dataset['theme'] = theme;

    // 更新Monaco编辑器主题
    const isDark = theme === 'dark' || (theme === 'auto' && systemTheme.matches);
    editor.updateOptions({ theme: isDark ? 'vs-dark' : 'vs' });
}
systemTheme.addEventListener('change', () => {
    const { theme } = getState();
    if (theme === 'auto') {
        applyTheme('auto');
    }
});

/** 初始化示例选择器 */
function initExampleSelector() {
    for (const [index, example] of EXAMPLES.entries()) {
        const option = document.createElement('option');
        option.value = index.toString();
        option.textContent = example.name;
        elExampleSelect.append(option);
    }

    elExampleSelect.addEventListener('change', () => {
        const selectedKey = Number.parseInt(elExampleSelect.value, 10);
        const example = EXAMPLES[selectedKey];
        if (!example) return;

        // 更新编辑器状态
        setState({ mode: example.mode, source: example.code });
        elModeSelect.value = example.mode;
        updateModel();
    });
}

/**
 * 初始化模式选择器
 */
function initModeSelector() {
    elModeSelect.value = getState().mode;
    elModeSelect.addEventListener('change', () => {
        setState({ mode: elModeSelect.value as InputMode });
        updateModel();
    });
}

/**
 * 初始化 tab 功能
 */
function initTabs() {
    const tabBtns = document.querySelectorAll<HTMLElement>('.tab-btn');
    const tabPanes = document.querySelectorAll<HTMLElement>('.tab-pane');

    for (const btn of Array.from(tabBtns)) {
        btn.addEventListener('click', () => {
            const targetTab = btn.dataset['tab'];

            // 更新按钮状态
            for (const b of Array.from(tabBtns)) {
                b.classList.remove('active');
            }
            btn.classList.add('active');

            // 更新面板状态
            for (const pane of Array.from(tabPanes)) {
                pane.classList.remove('active');
                if (pane.id === `${targetTab}-output`) {
                    pane.classList.add('active');
                }
            }
        });
    }
}

const overlay = monaco.utils.createOverflowWidgetsDomNode(elEditor);
const editor = monaco.editor.create(elEditor, {
    fontFamily: 'var(--code-font)',
    useShadowDOM: true,
    overflowWidgetsDomNode: overlay,
    formatOnType: true,
    formatOnPaste: true,
    fontLigatures: true,
    automaticLayout: true,
    wordWrap: 'on',
    wrappingIndent: 'indent',
    theme: systemTheme.matches ? 'vs-dark' : 'vs', // 初始主题，会在后面更新
    tabSize: 2,
    'semanticHighlighting.enabled': true,
    model: createModel(),
});

editor.onDidDispose(() => overlay.dispose());

// 初始化所有组件
setTimeout(() => {
    initThemeSelector();
    initExampleSelector();
    initModeSelector();
    initTabs();

    // 添加快捷键
    editor.addAction({
        id: 'SwitchMode',
        label: 'Switch Mode',
        keybindings: [KeyMod.CtrlCmd | KeyCode.KeyM],
        run: () => {
            const { mode } = getState();
            setState({ mode: mode === 'Script' ? 'Template' : 'Script' });
            elModeSelect.value = getState().mode;
            updateModel();
        },
    });

    editor.addAction({
        id: 'RunCode',
        label: 'Run Code',
        keybindings: [KeyMod.CtrlCmd | KeyCode.Enter],
        run: () => {
            void run();
        },
    });

    editor.onDidChangeModelContent(() => {
        setState({ source: editor.getValue() });
    });

    // 运行按钮事件
    elRunBtn.addEventListener('click', () => {
        void run();
    });
}, 0);

/** 编译 */
async function compileScript(): Promise<VmScript | undefined> {
    const compStart = performance.now();
    const { mode, source } = getState();
    try {
        const script = await compile(source, {
            pretty: true,
            input_mode: mode,
            sourceMap: true,
            fileName: 'playground.mira',
        });
        const compEnd = performance.now();
        consoleManager.info(`Compilation completed successfully in ${(compEnd - compStart).toFixed(3)}ms`);

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
        consoleManager.error(`Compilation failed in ${(compEnd - compStart).toFixed(3)}ms: ${errorText}`);
        return undefined;
    }
}

/** 运行 */
async function runScript(script: VmScript): Promise<void> {
    const execStart = performance.now();
    try {
        const execResult = script(globals);
        const execEnd = performance.now();
        const resultText = await print(execResult);
        elResultOutput.innerHTML = /*html*/ `
            <div class="section-title">Execution Result:</div>
            <div class="result-success">${resultText}</div>
        `;
        consoleManager.info(`Execution completed successfully in ${(execEnd - execStart).toFixed(3)}ms`);
    } catch (ex) {
        const execEnd = performance.now();
        const errorText = String(ex);
        elResultOutput.innerHTML = /*html*/ `
            <div class="section-title">Execution Error:</div>
            <div class="result-error">${errorText}</div>
        `;
        consoleManager.error(`Execution failed in ${(execEnd - execStart).toFixed(3)}ms: ${errorText}`);
    }
}

/** 编译运行 */
async function run() {
    if (elRunBtn.disabled) return;
    elRunBtn.disabled = true;

    consoleManager.clear();
    await consoleManager.render();
    consoleManager.resetTimer();

    try {
        const script = await compileScript();
        if (!script) return;
        await runScript(script);
    } finally {
        await consoleManager.render();

        elRunBtn.disabled = false;
    }
}
