import {
    Diagnostic,
    Disposable,
    languages,
    workspace,
    Location,
    DiagnosticSeverity,
    type DiagnosticCollection,
    type TextDocument,
    Uri,
    window,
    DiagnosticTag,
    DiagnosticRelatedInformation,
} from 'vscode';
import { createConfig, compileSync, ready } from '@mirascript/wasm';
import { DiagnosticCode } from '@mirascript/wasm/types';
import { getDiagnosticMessage, parseDiagnostics, type SourceDiagnostic } from '@mirascript/mirascript/subtle';
import { toRange } from '../adapter/utils.js';

await ready;
const configTemplate = createConfig({
    diagnostic_position_encoding: 'Utf16',
    diagnostic_tag: true,
    diagnostic_sourcemap: true,
    trivia: true,
    input_mode: 'Template',
});
const configScript = createConfig({
    diagnostic_position_encoding: 'Utf16',
    diagnostic_tag: true,
    diagnostic_sourcemap: true,
    trivia: true,
    input_mode: 'Script',
});

const disabledSchemes = new Set(['git', 'vsls', 'github', 'azurerepos', 'mirascript']);

/** 诊断信息 */
export class DiagnosticsManager extends Disposable {
    private readonly collection: DiagnosticCollection;
    private readonly diagVersionMap = new Map<string, number>();
    protected readonly disposables: Disposable[] = [];

    constructor() {
        super(() => {
            for (const disposable of this.disposables) {
                disposable.dispose();
            }
        });
        this.collection = languages.createDiagnosticCollection('mirascript');
        this.disposables.push(
            this.collection,
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

    /** 生成诊断 */
    private makeDiagnostic(
        model: TextDocument,
        diagnostic: SourceDiagnostic,
        severity: DiagnosticSeverity,
    ): Diagnostic {
        const { range, code } = diagnostic;
        const vsRange = toRange(range);
        let unnecessary = false;
        const deprecated = false;
        if (code === DiagnosticCode.UnusedLocalVariable || code === DiagnosticCode.UnusedLocalFunction) {
            unnecessary = true;
            severity = DiagnosticSeverity.Hint;
        }
        let message = getDiagnosticMessage(code) ?? 'Unknown error';
        if (message.includes(`$0`)) {
            message = message.replaceAll(`$0`, model.getText(vsRange));
        }
        const diag = new Diagnostic(vsRange, message, severity);
        diag.source = 'MiraScript';
        const codeName = DiagnosticCode[code];
        if (codeName) {
            diag.code = {
                value: codeName,
                target: Uri.parse(`https://mira.cloudpss.net/code/${codeName}`),
            };
        } else {
            diag.code = `${code}`;
        }
        if (unnecessary) {
            diag.tags ??= [];
            diag.tags.push(DiagnosticTag.Unnecessary);
        }
        if (deprecated) {
            diag.tags ??= [];
            diag.tags.push(DiagnosticTag.Deprecated);
        }
        if (diagnostic.references.length) {
            diag.relatedInformation = [];
            for (const ref of diagnostic.references) {
                const { range, code } = ref;
                const vsRange = toRange(range);
                const message = getDiagnosticMessage(code) ?? '...here';
                const refDiag = new DiagnosticRelatedInformation(new Location(model.uri, vsRange), message);
                diag.relatedInformation.push(refDiag);
            }
        }
        return diag;
    }
    /** 生成诊断 */
    private analyzeTextDocument(document: TextDocument): void {
        if (document.languageId !== 'mirascript' && document.languageId !== 'mirascript-template') {
            return;
        }
        if (disabledSchemes.has(document.uri.scheme)) {
            return;
        }
        const { version } = document;
        if (this.diagVersionMap.get(document.uri.toString()) === version) {
            return;
        }
        const source = document.getText();
        const config = document.languageId !== 'mirascript' ? configTemplate : configScript;
        const compiled = compileSync(source, config);
        if (version !== document.version) {
            return;
        }
        const parsed = parseDiagnostics(source, compiled.diagnostics);
        const errors = parsed.errors.map((d) => this.makeDiagnostic(document, d, DiagnosticSeverity.Error));
        const warnings = parsed.warnings.map((d) => this.makeDiagnostic(document, d, DiagnosticSeverity.Warning));
        const infos = parsed.infos.map((d) => this.makeDiagnostic(document, d, DiagnosticSeverity.Information));
        const hints = parsed.hints.map((d) => this.makeDiagnostic(document, d, DiagnosticSeverity.Hint));
        const markers = [...errors, ...warnings, ...infos, ...hints];
        this.diagVersionMap.set(document.uri.toString(), version);
        this.collection.set(document.uri, markers);
    }
}
