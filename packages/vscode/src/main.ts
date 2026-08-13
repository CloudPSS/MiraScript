import type * as vscode from 'vscode';
import type MarkdownIt from 'markdown-it';

/** API consumed by VS Code's built-in Markdown extension. */
interface ExtensionApi {
    /** Install MiraScript fenced-code highlighting. */
    extendMarkdownIt(markdownIt: MarkdownIt): MarkdownIt;
}

/** 激活扩展 */
export async function activate(context: vscode.ExtensionContext): Promise<ExtensionApi> {
    const [{ Scanner }, { ProvidersManager }, { ConfigManager }, markdownPreview] = await Promise.all([
        import('./lsp/scanner.js'),
        import('./lsp/providers.js'),
        import('./lsp/config.js'),
        import('./markdown-preview.js').then(async ({ MarkdownPreview }) => {
            const markdownPreview = new MarkdownPreview();
            await markdownPreview.initialize();
            return markdownPreview;
        }),
    ]);

    const configManager = new ConfigManager();
    const scanner = new Scanner();
    const providersManager = new ProvidersManager();

    context.subscriptions.push(configManager, scanner, providersManager, markdownPreview);
    return {
        extendMarkdownIt: (markdownIt) => markdownPreview.extendMarkdownIt(markdownIt),
    };
}

/** 扩展被禁用或卸载时调用 */
export function deactivate(): void {
    // 清理工作
}
