import { Disposable, workspace, type TextDocument, window } from 'vscode';
import { Provider } from '@mirascript/monaco/lsp';
import { ModelAdapter } from '../adapter/model.js';

/** 扫描工作区 */
export class Scanner extends Disposable {
    protected readonly disposables: Disposable[] = [];

    constructor() {
        super(() => {
            for (const disposable of this.disposables) {
                disposable.dispose();
            }
        });
        this.disposables.push(
            window.onDidChangeActiveTextEditor((editor) => {
                if (editor?.document) {
                    this.analyzeTextDocument(editor.document);
                }
            }),
            workspace.onDidChangeTextDocument((event) => {
                this.analyzeTextDocument(event.document);
            }),
            workspace.onDidOpenTextDocument((doc) => {
                this.analyzeTextDocument(doc);
            }),
            workspace.onDidChangeWorkspaceFolders((event) => {
                void this.scanWorkspace();
            }),
        );
        void this.init();
    }

    /** 初始化 */
    private async init(): Promise<void> {
        for (const editor of window.visibleTextEditors) {
            this.analyzeTextDocument(editor.document);
        }
        for (const doc of workspace.textDocuments) {
            this.analyzeTextDocument(doc);
        }
        await this.scanWorkspace();
    }

    /** 搜索工作区 */
    private async scanWorkspace(): Promise<void> {
        const files = await workspace.findFiles('**/*.{mira,miratpl}', '**/node_modules/**');
        for (const file of files) {
            await workspace.openTextDocument(file);
        }
    }

    /** 添加编译缓存 */
    private analyzeTextDocument(document: TextDocument): void {
        if (document.languageId !== 'mirascript' && document.languageId !== 'mirascript-template') {
            return;
        }
        void Provider.getCompileResult(ModelAdapter.from(document));
    }
}
