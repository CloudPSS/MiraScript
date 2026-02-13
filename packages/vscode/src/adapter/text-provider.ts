import { vscode, workspace, type Uri } from '#loader';

export const MIRA_TEXT_SCHEME = 'mirascript';
/**
 * 提供可修改的文本内容（只读）
 */
class MiraTextDocumentContentProvider implements vscode.TextDocumentContentProvider {
    private readonly content = new Map<string, string>();
    private readonly onDidChangeEmitter = new vscode.EventEmitter<Uri>();
    /** 设置文本内容 */
    setContent(uri: Uri, value: string | undefined): void {
        if (value == null) {
            this.content.delete(uri.toString());
        } else {
            this.content.set(uri.toString(), value);
        }
        this.onDidChangeEmitter.fire(uri);
    }
    readonly onDidChange = this.onDidChangeEmitter.event;
    /** @inheritdoc */
    provideTextDocumentContent(uri: Uri, token: vscode.CancellationToken): vscode.ProviderResult<string> {
        return this.content.get(uri.toString()) ?? '';
    }
}

export const MIRA_CONTENT_PROVIDER = new MiraTextDocumentContentProvider();
workspace.registerTextDocumentContentProvider(MIRA_TEXT_SCHEME, MIRA_CONTENT_PROVIDER);
