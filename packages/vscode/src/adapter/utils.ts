import * as monaco from '@private/monaco-editor/baseapi';
import type { languages as monacoLanguages, editor as monacoEditor } from '@private/monaco-editor';
import { vscode } from '#loader';
import { createAdapterFactory } from './base.js';

export enum CompletionItemInsertTextRule {
    None = 0,
    /**
     * Adjust whitespace/indentation of multiline insert texts to
     * match the current line indentation.
     */
    KeepWhitespace = 1,
    /**
     * `insertText` is a snippet.
     */
    InsertAsSnippet = 4,
}

/**
 * Converts a Monaco Editor range to a VS Code Range.
 */
export function toRange<T extends monaco.IRange | null | undefined>(
    range: T,
): T extends monaco.IRange ? vscode.Range : T {
    if (range == null) return range as never;
    if (range instanceof vscode.Range) return range as never;
    return new vscode.Range(
        range.startLineNumber - 1,
        range.startColumn - 1,
        range.endLineNumber - 1,
        range.endColumn - 1,
    ) as never;
}

/**
 * Converts a VS Code Range to a Monaco Editor range.
 */
export function fromRange(range: vscode.Range): monaco.Range {
    if (range instanceof monaco.Range) return range;
    return new monaco.Range(
        range.start.line + 1,
        range.start.character + 1,
        range.end.line + 1,
        range.end.character + 1,
    );
}

/**
 * Converts a Monaco Editor position to a VS Code Position.
 */
export function toPosition(position: monaco.IPosition): vscode.Position {
    if (position instanceof vscode.Position) return position;
    return new vscode.Position(position.lineNumber - 1, position.column - 1);
}

/**
 * Converts a VS Code Position to a Monaco Editor position.
 */
export function fromPosition(position: vscode.Position): monaco.Position {
    if (position instanceof monaco.Position) return position;
    return new monaco.Position(position.line + 1, position.character + 1);
}
/**
 * Converts a Monaco Editor TextEdit to a VS Code TextEdit.
 */
export function toTextEdit(edit: { range: monaco.IRange; text: string | null }): vscode.TextEdit {
    if (edit instanceof vscode.TextEdit) return edit;
    if (!edit.text) {
        return vscode.TextEdit.delete(toRange(edit.range));
    }
    return vscode.TextEdit.replace(toRange(edit.range), edit.text);
}
/**
 * Converts a Monaco Editor WorkspaceEdit to a VS Code WorkspaceEdit.
 */
export function toWorkspaceEdit(edit: monacoLanguages.WorkspaceEdit): vscode.WorkspaceEdit {
    const we = new vscode.WorkspaceEdit();
    if (edit.edits) {
        for (const e of edit.edits) {
            if ('redo' in e) {
                throw new Error('Cannot convert ICustomEdit to VS Code WorkspaceEdit.');
            }
            if ('resource' in e) {
                const uri = toUri(e.resource);
                we.replace(uri, toRange(e.textEdit.range), e.textEdit.text, e.metadata);
            } else {
                throw new Error('Cannot convert IWorkspaceFileEdit to VS Code WorkspaceEdit.');
            }
        }
    }
    return we;
}
/**
 * Converts a Monaco Editor IMarkdownString to a VS Code MarkdownString.
 */
export function toMarkdownString<const T extends monaco.IMarkdownString | string | undefined = monaco.IMarkdownString>(
    markdown: T,
): T extends monaco.IMarkdownString ? vscode.MarkdownString : T {
    if (markdown == null) return undefined as never;
    if (typeof markdown == 'string') return markdown as never;
    if (markdown instanceof vscode.MarkdownString) return markdown as never;
    const result = new vscode.MarkdownString(markdown.value);
    if (markdown.isTrusted !== undefined) {
        result.isTrusted = markdown.isTrusted;
    }
    if (markdown.supportThemeIcons !== undefined) {
        result.supportThemeIcons = markdown.supportThemeIcons;
    }
    return result as never;
}

/**
 * Converts a Monaco Editor Uri to a VS Code Uri.
 */
export function toUri<const T extends monaco.Uri | undefined = monaco.Uri>(
    uri: T,
): T extends monaco.Uri ? vscode.Uri : T {
    if (uri == null) return undefined as never;
    if (uri instanceof vscode.Uri) return uri as never;
    return vscode.Uri.parse(uri.toString()) as never;
}
/**
 * Converts a Monaco Editor Location to a VS Code Location.
 */
export function toLocation<const T extends monacoLanguages.Location | undefined = monacoLanguages.Location>(
    location: T,
): T extends monacoLanguages.Location ? vscode.Location : undefined {
    if (location == null) return undefined as never;
    if (location instanceof vscode.Location) return location as never;
    return new vscode.Location(toUri(location.uri), toRange(location.range)) as never;
}

/** Converts a Monaco Editor Command to a VS Code Command */
export function toCommand<const T extends monacoLanguages.Command | undefined = monacoLanguages.Command>(
    command: T,
): T extends monacoLanguages.Command ? vscode.Command : undefined {
    if (command == null) return undefined as never;
    if ('command' in command && 'title' in command) return command as never;
    return {
        title: command.title,
        command: command.id,
        arguments: command.arguments,
        tooltip: command.tooltip,
    } as never;
}

export const [toCompletionItem, fromCompletionItem] = createAdapterFactory<
    monacoLanguages.CompletionItem,
    vscode.CompletionItem
>(
    (item) => {
        return new vscode.CompletionItem(item.label);
    },
    (item, ci) => {
        ci.label = item.label;
        ci.kind = item.kind as unknown as vscode.CompletionItemKind;
        ci.tags = item.tags;
        ci.detail = item.detail;
        ci.documentation = toMarkdownString(item.documentation);
        ci.sortText = item.sortText;
        ci.filterText = item.filterText;
        ci.preselect = item.preselect;
        ci.insertText =
            item.insertTextRules === CompletionItemInsertTextRule.InsertAsSnippet
                ? new vscode.SnippetString(item.insertText)
                : item.insertText;
        const range =
            'insert' in item.range
                ? {
                      inserting: toRange(item.range.insert),
                      replacing: toRange(item.range.replace),
                  }
                : toRange(item.range);
        ci.range = range;
        ci.commitCharacters = item.commitCharacters;
        ci.additionalTextEdits = item.additionalTextEdits?.map(toTextEdit);
        ci.command = toCommand(item.command);
    },
);

export const [toDiagnosticRelatedInformation, fromDiagnosticRelatedInformation] = createAdapterFactory<
    monacoEditor.IRelatedInformation,
    vscode.DiagnosticRelatedInformation
>(
    (info) => {
        return new vscode.DiagnosticRelatedInformation(toLocation({ uri: info.resource, range: info }), info.message);
    },
    (info, dri) => {
        dri.location = toLocation({ uri: info.resource, range: info });
        dri.message = info.message;
    },
);

export const [toDiagnostic, fromDiagnostic] = createAdapterFactory<monacoEditor.IMarkerData, vscode.Diagnostic>(
    (marker) => {
        return new vscode.Diagnostic(toRange(marker), marker.message);
    },
    (marker, diagnostic) => {
        diagnostic.range = toRange(marker);
        diagnostic.message = marker.message;
        switch (marker.severity) {
            case monaco.MarkerSeverity.Error:
                diagnostic.severity = vscode.DiagnosticSeverity.Error;
                break;
            case monaco.MarkerSeverity.Warning:
                diagnostic.severity = vscode.DiagnosticSeverity.Warning;
                break;
            case monaco.MarkerSeverity.Info:
                diagnostic.severity = vscode.DiagnosticSeverity.Information;
                break;
            case monaco.MarkerSeverity.Hint:
                diagnostic.severity = vscode.DiagnosticSeverity.Hint;
                break;
        }
        diagnostic.source = marker.source;
        if (typeof marker.code == 'object') {
            diagnostic.code = {
                ...marker.code,
                target: toUri(marker.code.target),
            };
        } else {
            diagnostic.code = marker.code;
        }
        diagnostic.relatedInformation = marker.relatedInformation?.map(toDiagnosticRelatedInformation);
        diagnostic.tags = marker.tags?.map((t) => {
            switch (t) {
                case monaco.MarkerTag.Deprecated:
                    return vscode.DiagnosticTag.Deprecated;
                case monaco.MarkerTag.Unnecessary:
                    return vscode.DiagnosticTag.Unnecessary;
            }
        });
    },
);

export const [toCodeAction, fromCodeAction] = createAdapterFactory<monacoLanguages.CodeAction, vscode.CodeAction>(
    (action) => {
        return new vscode.CodeAction(action.title);
    },
    (action, ca) => {
        switch (action.kind) {
            case 'quickfix':
                ca.kind = vscode.CodeActionKind.QuickFix;
                break;
            case 'refactor':
                ca.kind = vscode.CodeActionKind.Refactor;
                break;
            case 'refactor.extract':
                ca.kind = vscode.CodeActionKind.RefactorExtract;
                break;
            case 'refactor.inline':
                ca.kind = vscode.CodeActionKind.RefactorInline;
                break;
            case 'refactor.move':
                ca.kind = vscode.CodeActionKind.RefactorMove;
                break;
            case 'refactor.rewrite':
                ca.kind = vscode.CodeActionKind.RefactorRewrite;
                break;
            case 'source':
                ca.kind = vscode.CodeActionKind.Source;
                break;
            case 'source.organizeImports':
                ca.kind = vscode.CodeActionKind.SourceOrganizeImports;
                break;
            case 'source.fixAll':
                ca.kind = vscode.CodeActionKind.SourceFixAll;
                break;
            case undefined:
            default:
                break;
        }
        if (action.edit) ca.edit = toWorkspaceEdit(action.edit);
        if (action.diagnostics) ca.diagnostics = action.diagnostics.map(toDiagnostic);
        if (action.command) throw new Error('Cannot convert CodeAction with command to VS Code CodeAction.');
        ca.isPreferred = action.isPreferred;
        ca.disabled = action.disabled ? { reason: action.disabled } : undefined;
    },
);

export const [toCodeLens, fromCodeLens] = createAdapterFactory<monacoLanguages.CodeLens, vscode.CodeLens>(
    (lens) => new vscode.CodeLens(toRange(lens.range)),
    (lens, cl) => {
        cl.range = toRange(lens.range);
        cl.command = toCommand(lens.command);
    },
);

export const [toInlayHintLabelPart, fromInlayHintLabelPart] = createAdapterFactory<
    monacoLanguages.InlayHintLabelPart,
    vscode.InlayHintLabelPart
>(
    (part) => new vscode.InlayHintLabelPart(part.label),
    (part, lp) => {
        lp.value = part.label;
        lp.tooltip = toMarkdownString(part.tooltip);
        lp.location = toLocation(part.location);
        lp.command = toCommand(part.command);
    },
);

export const [toInlayHint, fromInlayHint] = createAdapterFactory<monacoLanguages.InlayHint, vscode.InlayHint>(
    (item) =>
        new vscode.InlayHint(
            toPosition(item.position),
            typeof item.label === 'string' ? item.label : item.label.map(toInlayHintLabelPart),
        ),
    (item, h) => {
        h.label = typeof item.label === 'string' ? item.label : item.label.map(toInlayHintLabelPart);
        h.position = toPosition(item.position);
        h.kind = item.kind;
        h.paddingLeft = item.paddingLeft;
        h.paddingRight = item.paddingRight;
        h.tooltip = toMarkdownString(item.tooltip);
        h.textEdits = item.textEdits?.map(toTextEdit);
    },
);

export const [toParameterInformation, fromParameterInformation] = createAdapterFactory<
    monacoLanguages.ParameterInformation,
    vscode.ParameterInformation
>(
    (param) => new vscode.ParameterInformation(param.label),
    (param, pi) => {
        pi.label = param.label;
        pi.documentation = toMarkdownString(param.documentation);
    },
);

export const [toSignatureInformation, fromSignatureInformation] = createAdapterFactory<
    monacoLanguages.SignatureInformation,
    vscode.SignatureInformation
>(
    (sig) => new vscode.SignatureInformation(sig.label),
    (sig, si) => {
        si.label = sig.label;
        si.documentation = toMarkdownString(sig.documentation);
        si.parameters = sig.parameters?.map(toParameterInformation);
    },
);

export const [toSignatureHelp, fromSignatureHelp] = createAdapterFactory<
    monacoLanguages.SignatureHelp,
    vscode.SignatureHelp
>(
    () => new vscode.SignatureHelp(),
    (sh, s) => {
        s.signatures = sh.signatures.map(toSignatureInformation);
        s.activeParameter = sh.activeParameter;
        s.activeSignature = sh.activeSignature;
    },
);

export const [toDocumentSymbol, fromDocumentSymbol] = createAdapterFactory<
    monacoLanguages.DocumentSymbol,
    vscode.DocumentSymbol
>(
    (symbol) =>
        new vscode.DocumentSymbol(
            symbol.name,
            symbol.detail,
            symbol.kind,
            toRange(symbol.range),
            toRange(symbol.selectionRange),
        ),
    (symbol, ds) => {
        ds.name = symbol.name;
        ds.detail = symbol.detail;
        ds.kind = symbol.kind;
        ds.range = toRange(symbol.range);
        ds.selectionRange = toRange(symbol.selectionRange);
        ds.children = symbol.children?.map(toDocumentSymbol) ?? [];
    },
);
