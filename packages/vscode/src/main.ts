import type * as vscode from 'vscode';
import { DiagnosticsManager } from './lsp/diagnostics.js';

/** 激活扩展 */
export function activate(context: vscode.ExtensionContext): void {
    context.subscriptions.push(new DiagnosticsManager());
}

/** 扩展被禁用或卸载时调用 */
export function deactivate(): void {
    // 清理工作
}
