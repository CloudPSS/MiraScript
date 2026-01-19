import type * as vscode from 'vscode';

/** 激活扩展 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    const { Scanner } = await import('./lsp/scanner.js');
    const { ProvidersManager } = await import('./lsp/providers.js');
    const { ConfigManager } = await import('./lsp/config.js');

    const configManager = new ConfigManager();
    const scanner = new Scanner();
    const providersManager = new ProvidersManager();

    context.subscriptions.push(configManager, scanner, providersManager);
}

/** 扩展被禁用或卸载时调用 */
export function deactivate(): void {
    // 清理工作
}
