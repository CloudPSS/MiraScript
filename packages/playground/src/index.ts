/* eslint-disable no-console */
import './index.css';
import * as monaco from '@private/monaco-editor';
import { KeyCode, KeyMod } from '@private/monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';
import {
    type VmAny,
    VmExtern,
    createVmGlobal,
    isVmExtern,
    isVmModule,
    compile,
    serialize,
    VmModule,
    VmFunction,
    type InputMode,
} from 'mirascript';

// 加载 javascript 模型以便语法高亮
monaco.editor.createModel('', 'javascript').dispose();

// 使用 Vite 的 import.meta.glob 动态加载示例文件
const exampleModules = import.meta.glob('./*.{mira,miratpl}', {
    base: '../../../examples',
    query: '?raw',
    import: 'default',
    eager: true,
});

// 将文件路径转换为示例数据
const examples: Record<string, { name: string; mode: InputMode; code: string }> = {};

for (const [path, content] of Object.entries(exampleModules)) {
    const filename = path.split('/').pop()!;
    const isTemplate = filename.endsWith('.miratpl');

    // 提取序号和名称
    const regex = /^(\d{2})_(.+)\.(mira|miratpl)$/;
    const match = regex.exec(filename);
    if (!match || match.length < 4) {
        console.warn(`文件名格式不正确: ${filename}`);
        continue;
    }

    const baseName = match[2]!;

    // 生成友好的显示名称
    const displayName = baseName
        .split('_')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');

    examples[baseName] = {
        name: displayName, // 不包含序号，只显示友好名称
        mode: isTemplate ? 'Template' : 'Script',
        code: content as string,
    };
}

// 添加默认示例以防没有文件被加载
if (Object.keys(examples).length === 0) {
    examples['01_hello_world.mira'] = {
        name: 'Hello World',
        mode: 'Script',
        code: `debug_print("Hello, World!");`,
    };
}

/** 管理控制台输出的类 */
class ConsoleManager {
    private entries: Array<{
        type: 'log' | 'error' | 'warn' | 'info';
        message: Promise<string> | string;
        timestamp: Date;
    }> = [];
    private readonly outputElement: HTMLElement;

    constructor(outputElement: HTMLElement) {
        this.outputElement = outputElement;
    }

    /** 添加日志消息 */
    log(message: Promise<string> | string) {
        this.addEntry('log', message);
    }

    /** 添加错误消息 */
    error(message: Promise<string> | string) {
        this.addEntry('error', message);
    }

    /** 添加警告消息 */
    warn(message: Promise<string> | string) {
        this.addEntry('warn', message);
    }

    /** 添加信息消息 */
    info(message: Promise<string> | string) {
        this.addEntry('info', message);
    }

    /** 添加条目到控制台 */
    private addEntry(type: 'log' | 'error' | 'warn' | 'info', message: Promise<string> | string) {
        const entry = {
            type,
            message,
            timestamp: new Date(),
        };
        this.entries.push(entry);
        void this.render();
    }

    /** 清空控制台 */
    clear() {
        this.entries = [];
        void this.render();
    }

    /** 渲染控制台内容 */
    private async render() {
        const htmlArray = this.entries.map(async (entry) => {
            const time = entry.timestamp.toLocaleTimeString(undefined, {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                fractionalSecondDigits: 3,
            });
            return /* html */ `<div class="console-entry ${entry.type}">
                <time class="console-time" datetime=${entry.timestamp.toISOString()}>[${time}]</time>
                <span class="console-message">${await entry.message}</span>
            </div>`;
        });

        const html = await Promise.all(htmlArray).then((lines) => lines.join(''));
        this.outputElement.innerHTML = html;
        this.outputElement.scrollTop = this.outputElement.scrollHeight;
    }
}

const arr = [1, 2, [1, 2], { x: 0 }];
arr[100] = 100; // make a sparse array

// 初始化控制台管理器
const consoleOutput = document.querySelector<HTMLDivElement>('#console-output')!;
const consoleManager = new ConsoleManager(consoleOutput);

/** 创建简单的debug_print函数 */
function debugPrint(...args: VmAny[]) {
    const messages = args.map(async (arg) => {
        if (typeof arg === 'string') return arg;
        return print(arg);
    });
    consoleManager.log(Promise.all(messages).then((message) => message.join(' ')));
}

const globals = createVmGlobal(
    {
        extern_arr: new VmExtern(arr),
        obj: { a: [], b: 1, c: '2', d: { e: 3 } },
        arr: [1, 2, 3],
        long_str: 'Long string content'.repeat(10000),
        mod: new VmModule('test', {
            sin: createVmGlobal().sin,
            inner: new VmModule('inner', {}),
        }),
        debug_print: VmFunction(debugPrint, {
            fullName: 'debug_print',
            summary: 'Print debug messages to console',
        }),
        name: 'MiraScript', // for template examples
    },
    {
        extern_obj: { a: [], b: 1, c: '2', d: { e: 3 }, sin: createVmGlobal().sin },
        globalThis,
    },
);

registerMiraScript(monaco, () => globals);

let mode: InputMode = (localStorage.getItem('mode') as InputMode) || 'Script';

let modelIndex = 1;
const createModel = (value: string) => {
    // 使用唯一的URI避免冲突
    const uri = monaco.Uri.parse(`file:///playground-${modelIndex++}.${mode === 'Template' ? 'miratpl' : 'mira'}`);
    const model = monaco.editor.createModel(value, mode === 'Template' ? 'mirascript-template' : 'mirascript', uri);
    model.setEOL(monaco.editor.EndOfLineSequence.LF);
    return model;
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
    // 将示例按序号排序
    const sortedExamples = Object.entries(examples).sort(([a], [b]) => {
        // 提取序号进行排序
        const regexOrder = /^(\d{2})/;
        const matchA = regexOrder.exec(a);
        const matchB = regexOrder.exec(b);
        const orderA = Number.parseInt(matchA?.[1] || '99', 10);
        const orderB = Number.parseInt(matchB?.[1] || '99', 10);
        return orderA - orderB;
    });

    for (const [key, example] of sortedExamples) {
        const option = document.createElement('option');
        option.value = key;
        option.textContent = example.name;
        elExampleSelect.append(option);
    }

    elExampleSelect.addEventListener('change', () => {
        const selectedKey = elExampleSelect.value;
        if (selectedKey && examples[selectedKey]) {
            const example = examples[selectedKey];

            // 更新模式
            mode = example.mode;
            elModeSelect.value = mode;
            localStorage.setItem('mode', mode);

            // 创建新模型前先销毁旧模型
            const oldModel = editor.getModel();
            const newModel = createModel(example.code);
            editor.setModel(newModel);
            oldModel?.dispose();

            // 保存到本地存储
            localStorage.setItem('source', example.code);
        }
    });
}

// 初始化模式选择器
/**
 * Initialize the mode selector dropdown with current mode
 */
function initModeSelector() {
    elModeSelect.value = mode;
    elModeSelect.addEventListener('change', () => {
        mode = elModeSelect.value as InputMode;
        localStorage.setItem('mode', mode);

        // 创建新模型前先销毁旧模型
        const oldModel = editor.getModel();
        const newModel = createModel(editor.getValue());
        editor.setModel(newModel);
        oldModel?.dispose();
    });
}

/**
 * Initialize tab functionality for output panels
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

const value =
    localStorage.getItem('source') || examples['01_hello_world.mira']?.code || `debug_print("Hello, World!");`;
const overlay = monaco.utils.createOverflowWidgetsDomNode(elEditor);
const editor = monaco.editor.create(elEditor, {
    fontFamily: 'Sarasa Mono SC, monospace',
    useShadowDOM: true,
    overflowWidgetsDomNode: overlay,
    formatOnType: true,
    formatOnPaste: true,
    fontLigatures: true,
    automaticLayout: true,
    wordWrap: 'on',
    wrappingIndent: 'indent',
    theme: 'vs-dark',
    tabSize: 2,
    'semanticHighlighting.enabled': true,
    model: createModel(value),
});

editor.onDidDispose(() => overlay.dispose());

// 初始化所有组件
setTimeout(() => {
    initExampleSelector();
    initModeSelector();
    initTabs();

    // 添加快捷键
    editor.addAction({
        id: 'SwitchMode',
        label: 'Switch Mode',
        keybindings: [KeyMod.CtrlCmd | KeyCode.KeyM],
        run: () => {
            mode = mode === 'Script' ? 'Template' : 'Script';
            elModeSelect.value = mode;
            localStorage.setItem('mode', mode);

            // 创建新模型前先销毁旧模型
            const oldModel = editor.getModel();
            const newModel = createModel(editor.getValue());
            editor.setModel(newModel);
            oldModel?.dispose();
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
        localStorage.setItem('source', editor.getValue());
    });

    // 运行按钮事件
    elRunBtn.addEventListener('click', () => {
        void run();
    });
}, 1);

/** 语法高亮 */
async function syntaxHighlight(value: string, languageId: string): Promise<string> {
    let highlighted = await monaco.editor.colorize(value, languageId, {});
    if (highlighted.endsWith('<br/>') && !value.endsWith('\n')) {
        // 如果高亮结果以 <br/> 结尾且原始值没有换行符，则去掉 <br/>
        highlighted = highlighted.slice(0, -'<br/>'.length);
    }
    highlighted = highlighted.replaceAll(/(\u00A0)+/g, '$&<wbr>');
    return highlighted;
}

/** 将值转为显示 */
async function print(value: VmAny | Error): Promise<string> {
    if (value === null) return syntaxHighlight('nil', 'mirascript');
    if (value === undefined) return '<uninitialized>';
    if (value instanceof Error) return value.toString();
    if (typeof value == 'function') {
        return syntaxHighlight(String(value), 'javascript');
    }
    if (isVmExtern(value) || isVmModule(value) || typeof value == 'function') {
        return String(value);
    }
    return syntaxHighlight(serialize(value), 'mirascript');
}

/** 编译运行 */
async function run() {
    const value = editor.getValue();

    // 清空控制台
    consoleManager.clear();

    // 禁用运行按钮
    elRunBtn.disabled = true;
    elRunBtn.textContent = 'Running...';

    try {
        console.time('transpile');
        const result = await compile(value, {
            pretty: true,
            input_mode: mode,
            sourceMap: true,
            fileName: 'playground.mira',
        }).finally(() => {
            console.timeEnd('transpile');
        });

        // 显示编译结果
        const compiledCode = result.toString();
        const highlightedJS = await syntaxHighlight(compiledCode, 'javascript');
        elCompiledOutput.innerHTML = /*html*/ `
            <div class="section-title">Compiled JavaScript:</div>
            <div class="compiled-code">${highlightedJS}</div>
        `;

        console.time('execute');
        try {
            const execResult = result(globals);
            const resultText = await print(execResult);
            elResultOutput.innerHTML = /*html*/ `
                <div class="section-title">Execution Result:</div>
                <div class="result-success"><pre>${resultText}</pre></div>
            `;

            consoleManager.info(`Execution completed successfully`);
        } catch (ex) {
            const errorText = String(ex);
            elResultOutput.innerHTML = /*html*/ `
                <div class="section-title">Execution Error:</div>
                <div class="result-error"><pre>${errorText}</pre></div>
            `;
            consoleManager.error(`Execution failed: ${errorText}`);
        }
        console.timeEnd('execute');
    } catch (ex) {
        const errorText = String(ex);
        elCompiledOutput.innerHTML = /*html*/ `
            <div class="section-title">Compilation Error:</div>
            <div class="result-error"><pre>${errorText}</pre></div>
        `;
        elResultOutput.innerHTML = /*html*/ `
            <div class="section-title">Execution Result:</div>
            <div class="result-error">Compilation failed</div>
        `;
        consoleManager.error(`Compilation failed: ${errorText}`);
    } finally {
        // 重新启用运行按钮
        elRunBtn.disabled = false;
        elRunBtn.textContent = 'Run (Ctrl+Enter)';
    }
}
