import './index.css';
import './theme.js';
import type { InputMode } from '@mirascript/mirascript';
import { EXAMPLES } from './examples.js';
import { getState, setState } from './state-manager.js';
import { resultManager } from './result-manager.js';
import { monaco, globals, mirascript, consoleManager, ready } from './loader.js';
import { createEditor } from './monaco.js';

if (!location.pathname.endsWith('/')) {
    location.pathname += '/';
}

await ready;

const createModel = () => {
    const { source, mode } = getState();
    const uri = monaco.Uri.parse(`file:///playground`);
    const model = monaco.editor.createModel(source, mode === 'Template' ? 'mirascript-template' : 'mirascript', uri);
    model.setEOL(monaco.editor.EndOfLineSequence.LF);
    return model;
};

const updateModel = () => {
    const oldModel = editor.getModel();
    if (!oldModel) return;
    const { source, mode } = getState();
    const language = mode === 'Template' ? 'mirascript-template' : 'mirascript';
    if (oldModel.getLanguageId() !== language) {
        monaco.editor.setModelLanguage(oldModel, language);
    }
    if (oldModel.getValue() !== source) {
        oldModel.setValue(source);
    }
};

// UI 元素
const elEditor = document.querySelector<HTMLDivElement>('#editor')!;
const elExampleSelect = document.querySelector<HTMLSelectElement>('#example-select')!;
const elModeSelect = document.querySelector<HTMLSelectElement>('#mode-select')!;
const elRunBtn = document.querySelector<HTMLButtonElement>('#run-btn')!;
const elCompiledOutput = document.querySelector<HTMLDivElement>('#compiled-output')!;
const elResultOutput = document.querySelector<HTMLDivElement>('#result-output')!;

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

        void example.code().then((code) => {
            // 更新编辑器状态
            setState({ mode: example.mode, source: code });
            elModeSelect.value = example.mode;
            updateModel();
        });
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

    for (const btn of tabBtns) {
        btn.addEventListener('click', () => {
            const targetTab = btn.dataset['tab'];

            // 更新按钮状态
            for (const b of tabBtns) {
                b.classList.remove('active');
            }
            btn.classList.add('active');

            // 更新面板状态
            for (const pane of tabPanes) {
                pane.classList.remove('active');
                if (pane.id === `${targetTab}-output`) {
                    pane.classList.add('active');
                }
            }
        });
    }
}

const editor = createEditor(elEditor, {
    wordWrap: 'on',
    wrappingIndent: 'indent',
    formatOnType: true,
    formatOnPaste: true,
    minimap: { renderCharacters: false },
    model: createModel(),
});

// 初始化所有组件
setTimeout(() => {
    initExampleSelector();
    initModeSelector();
    initTabs();

    // 添加快捷键
    editor.addAction({
        id: 'SwitchMode',
        label: 'Switch Mode',
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyM],
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
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
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

const compileAndRun = resultManager(consoleManager, elCompiledOutput, elResultOutput, globals);

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
        try {
            mirascript.configCheckpoint(Number.POSITIVE_INFINITY);
            return await run();
        } finally {
            mirascript.configCheckpoint(500);
        }
    },
});
