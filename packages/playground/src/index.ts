import './index.css';
import * as monaco from '@private/monaco-editor';
import { KeyCode, KeyMod } from '@private/monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';
import { configCheckpoint, type InputMode } from '@mirascript/mirascript';
import * as mirascript from '@mirascript/mirascript';
import * as mirascriptSubtle from '@mirascript/mirascript/subtle';
import { ConsoleManager } from './console-manager.js';
import { EXAMPLES } from './examples.js';
import { getState, setState, type ThemeMode } from './state-manager.js';
import { globals } from './globals.js';
import { resultManager } from './result-manager.js';

// 暴露到全局以便调试
Object.defineProperty(globalThis, 'mirascript', {
    value: Object.freeze({
        __proto__: null,
        ...mirascript,
        subtle: Object.freeze({
            __proto__: null,
            ...mirascriptSubtle,
        }),
    }),
});

// 初始化控制台管理器
const consoleManager = new ConsoleManager(document.querySelector<HTMLDivElement>('#console-output')!);

const g = globals(consoleManager);
registerMiraScript(monaco, () => g);

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
    minimap: { renderCharacters: false },
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

const compileAndRun = resultManager(consoleManager, elCompiledOutput, elResultOutput, g);

configCheckpoint(500);
/** 编译运行 */
async function run() {
    if (elRunBtn.disabled) return;
    elRunBtn.disabled = true;

    consoleManager.clear();
    await consoleManager.render();
    consoleManager.resetTimer();

    try {
        return await compileAndRun();
    } finally {
        await consoleManager.render();

        elRunBtn.disabled = false;
    }
}

Object.defineProperty(globalThis, 'playgroundRun', {
    value: async () => {
        configCheckpoint(Number.POSITIVE_INFINITY);
        const ret = await run();
        configCheckpoint(500);
        return ret;
    },
});
