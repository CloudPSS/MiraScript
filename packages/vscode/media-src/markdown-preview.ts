let init = async () => {
    const loader = Promise.resolve().then(async () => {
        const monaco = await import('@private/monaco-editor/api');
        await import('@private/monaco-editor/theme');
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

/** 高亮元素 */
async function highlightElement(el: HTMLElement) {
    const monaco = await init();
    const isMiratpl = el.classList.contains('language-miratpl');
    if (el.textContent?.endsWith('\n')) {
        el.textContent = el.textContent.slice(0, -1);
    }
    await monaco.editor.colorizeElement(el, {
        theme: 'vs',
        tabSize: 4,
        mimeType: isMiratpl ? 'text/x-mirascript-template' : 'text/x-mirascript',
    });
}

/**
 * Main
 */
function main() {
    const elMira = Array.from(document.querySelectorAll<HTMLElement>('code.language-mira, code.language-miratpl'));
    if (elMira.length === 0) {
        return;
    }
    for (const el of elMira) {
        void highlightElement(el);
    }
}

main();
