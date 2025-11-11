let init = async () => {
    const loader = Promise.resolve().then(async () => {
        const monaco = await import('@private/monaco-editor/api');
        await import('@private/monaco-editor/themes');
        const { registerMonacoApi, registerContribution } = await import('@mirascript/monaco');
        registerMonacoApi(monaco);
        registerContribution();
        const { registerBasic } = await import('@mirascript/monaco/basic');
        await registerBasic();
        return monaco;
    });
    init = async () => loader; // 重置 init 函数，避免重复加载
    return loader;
};

const themes = ['vs', 'vs-dark', 'hc-black', 'hc-light'];
/** 高亮元素 */
async function highlightElement(monaco: typeof import('@private/monaco-editor/api'), el: HTMLElement) {
    if (el.classList.contains(theme)) {
        return;
    }
    const { code } = el.dataset;
    if (code) {
        el.classList.remove(...themes);
        el.textContent = code;
    } else {
        el.dataset['code'] = el.textContent;
    }
    const isMiratpl = el.classList.contains('language-miratpl');
    if (el.textContent?.endsWith('\n')) {
        el.textContent = el.textContent.slice(0, -1);
    }
    await monaco.editor.colorizeElement(el, {
        theme,
        tabSize: 4,
        mimeType: isMiratpl ? 'text/x-mirascript-template' : 'text/x-mirascript',
    });
}

let theme = getTheme();
/** 当前颜色模式 */
function getTheme() {
    const { body } = document;
    if (body.classList.contains('vscode-dark')) {
        return 'vs-dark';
    }
    if (body.classList.contains('vscode-high-contrast')) {
        if (body.classList.contains('vscode-high-contrast-light')) {
            return 'hc-light';
        }
        return 'hc-black';
    }
    return 'vs';
}

const SELECTOR = 'code.language-mira, code.language-miratpl';

/** 重新高亮 */
async function highlight(nodes: Iterable<HTMLElement> | null = null) {
    const elMira = nodes ? Array.from(nodes) : Array.from(document.querySelectorAll<HTMLElement>(SELECTOR));
    if (elMira.length === 0) {
        return;
    }
    const monaco = await init();
    await Promise.all(elMira.map(async (el) => highlightElement(monaco, el)));
}

/**
 * Main
 */
async function main() {
    await highlight();
    // 观测编辑
    new MutationObserver((mutations) => {
        const nodes = new Set<HTMLElement>();
        for (const mutation of mutations) {
            const node = mutation.target;
            if (node instanceof HTMLElement) {
                if (node.matches(SELECTOR)) {
                    nodes.add(node);
                } else {
                    const els = node.querySelectorAll<HTMLElement>(SELECTOR);
                    for (const el of Array.from(els)) nodes.add(el);
                }
            }
        }
        void highlight(nodes);
    }).observe(document.body, {
        childList: true,
        subtree: true,
    });
    // 观测主题变更
    new MutationObserver(() => {
        const newTheme = getTheme();
        if (newTheme !== theme) {
            theme = newTheme;
            void highlight();
        }
    }).observe(document.body, {
        attributes: true,
        attributeFilter: ['class'],
    });
}

requestIdleCallback(() => void main());
