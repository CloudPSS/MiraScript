/**
 * Main
 */
async function main() {
    const monaco = await import('@private/monaco-editor/api');
    await import('@private/monaco-editor/theme');
    const { registerMonacoApi, register } = await import('@mirascript/monaco');
    registerMonacoApi(monaco);
    register();
    const { registerBasic } = await import('@mirascript/monaco/basic');
    await registerBasic();
    for (const el of Array.from(document.querySelectorAll('code.language-mira'))) {
        await monaco.editor.colorizeElement(el as HTMLElement, {
            theme: 'vs',
            tabSize: 4,
            mimeType: 'text/x-mirascript',
        });
    }
}

void main();
