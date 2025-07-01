import type * as vscode from 'vscode';

/** 激活扩展 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    const { DiagnosticsManager } = await import('./lsp/diagnostics.js');
    const { ProvidersManager } = await import('./lsp/providers.js');
    context.subscriptions.push(new DiagnosticsManager(), new ProvidersManager());
}

/** 扩展被禁用或卸载时调用 */
export function deactivate(): void {
    // 清理工作
}
