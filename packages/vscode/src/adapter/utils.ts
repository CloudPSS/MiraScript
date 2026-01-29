import {
    type IRange,
    type IPosition,
    Position as MonacoPosition,
    Range as MonacoRange,
    type IMarkdownString,
    type Uri as MonacoUri,
    MarkerSeverity,
    MarkerTag,
} from '@private/monaco-editor/baseapi';
import type { languages as monacoLanguages, editor as monacoEditor } from '@private/monaco-editor';
import {
    Diagnostic,
    Range as VsRange,
    Position as VsPosition,
    TextEdit as VsTextEdit,
    WorkspaceEdit as VsWorkspaceEdit,
    MarkdownString as VsMarkdownString,
    Location as VsLocation,
    Uri as VsUri,
    type CompletionItem as VsCompletionItem,
    type CompletionItemKind,
    SnippetString,
    DiagnosticRelatedInformation,
    DiagnosticTag,
    DiagnosticSeverity,
    CodeAction,
    CodeActionKind,
} from 'vscode';

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
export function toRange<T extends IRange | null | undefined>(range: T): T extends IRange ? VsRange : T {
    if (range == null) return range as never;
    if (range instanceof VsRange) return range as never;
    return new VsRange(
        range.startLineNumber - 1,
        range.startColumn - 1,
        range.endLineNumber - 1,
        range.endColumn - 1,
    ) as never;
}

/**
 * Converts a VS Code Range to a Monaco Editor range.
 */
export function fromRange(range: VsRange): MonacoRange {
    if (range instanceof MonacoRange) return range;
    return new MonacoRange(
        range.start.line + 1,
        range.start.character + 1,
        range.end.line + 1,
        range.end.character + 1,
    );
}

/**
 * Converts a Monaco Editor position to a VS Code Position.
 */
export function toPosition(position: IPosition): VsPosition {
    if (position instanceof VsPosition) return position;
    return new VsPosition(position.lineNumber - 1, position.column - 1);
}

/**
 * Converts a VS Code Position to a Monaco Editor position.
 */
export function fromPosition(position: VsPosition): MonacoPosition {
    if (position instanceof MonacoPosition) return position;
    return new MonacoPosition(position.line + 1, position.character + 1);
}
/**
 * Converts a Monaco Editor TextEdit to a VS Code TextEdit.
 */
export function toTextEdit(edit: { range: IRange; text: string | null }): VsTextEdit {
    if (edit instanceof VsTextEdit) return edit;
    if (!edit.text) {
        return VsTextEdit.delete(toRange(edit.range));
    }
    return VsTextEdit.replace(toRange(edit.range), edit.text);
}
/**
 * Converts a Monaco Editor WorkspaceEdit to a VS Code WorkspaceEdit.
 */
export function toWorkspaceEdit(edit: monacoLanguages.WorkspaceEdit): VsWorkspaceEdit {
    const we = new VsWorkspaceEdit();
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
export function toMarkdownString<const T extends IMarkdownString | string | undefined = IMarkdownString>(
    markdown: T,
): T extends IMarkdownString ? VsMarkdownString : T {
    if (markdown == null) return undefined as never;
    if (typeof markdown == 'string') return markdown as never;
    if (markdown instanceof VsMarkdownString) return markdown as never;
    const result = new VsMarkdownString(markdown.value);
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
export function toUri<const T extends MonacoUri | undefined = MonacoUri>(uri: T): T extends MonacoUri ? VsUri : T {
    if (uri == null) return undefined as never;
    if (uri instanceof VsUri) return uri as never;
    return VsUri.parse(uri.toString()) as never;
}
/** Monaco Editor Location */
type MonacoLocation = { uri: MonacoUri; range: IRange };
/**
 * Converts a Monaco Editor Location to a VS Code Location.
 */
export function toLocation<const T extends MonacoLocation | undefined = MonacoLocation>(
    location: T,
): T extends MonacoLocation ? VsLocation : undefined {
    if (location == null) return undefined as never;
    if (location instanceof VsLocation) return location as never;
    return new VsLocation(toUri(location.uri), toRange(location.range)) as never;
}
/**
 * Convert to VS Code CompletionItem
 */
export function toCompletionItem(
    item: Omit<monacoLanguages.CompletionItem, 'kind'> & { kind: number },
): VsCompletionItem & { original: monacoLanguages.CompletionItem } {
    const range =
        'insert' in item.range
            ? {
                  inserting: toRange(item.range.insert),
                  replacing: toRange(item.range.replace),
              }
            : toRange(item.range);
    const ci: VsCompletionItem & { original: monacoLanguages.CompletionItem } = {
        label: item.label,
        kind: item.kind as CompletionItemKind,
        tags: item.tags,
        detail: item.detail,
        documentation: toMarkdownString(item.documentation),
        sortText: item.sortText,
        filterText: item.filterText,
        preselect: item.preselect,
        insertText:
            item.insertTextRules === CompletionItemInsertTextRule.InsertAsSnippet
                ? new SnippetString(item.insertText)
                : item.insertText,
        range,
        commitCharacters: item.commitCharacters,
        additionalTextEdits: item.additionalTextEdits?.map(toTextEdit),
        command: undefined,
        original: item,
    };
    return ci;
}

/** Converted Diagnostics */
const MonacoDiagnostics = new WeakMap<monacoEditor.IMarkerData, MonacoDiagnostic>();
/** Monaco Diagnostic adapter */
class MonacoDiagnostic extends Diagnostic {
    constructor(readonly marker: monacoEditor.IMarkerData) {
        super(toRange(marker), marker.message);
        switch (marker.severity as number as MarkerSeverity) {
            case MarkerSeverity.Error:
                this.severity = DiagnosticSeverity.Error;
                break;
            case MarkerSeverity.Warning:
                this.severity = DiagnosticSeverity.Warning;
                break;
            case MarkerSeverity.Info:
                this.severity = DiagnosticSeverity.Information;
                break;
            case MarkerSeverity.Hint:
                this.severity = DiagnosticSeverity.Hint;
                break;
        }
        this.source = marker.source;
        if (typeof marker.code == 'object') {
            this.code = {
                ...marker.code,
                target: toUri(marker.code.target),
            };
        } else {
            this.code = marker.code;
        }
        this.relatedInformation = marker.relatedInformation?.map(
            (i) => new DiagnosticRelatedInformation(toLocation({ uri: i.resource, range: i }), i.message),
        );
        this.tags = marker.tags?.map((t) => {
            switch (t as number as MarkerTag) {
                case MarkerTag.Deprecated:
                    return DiagnosticTag.Deprecated;
                case MarkerTag.Unnecessary:
                    return DiagnosticTag.Unnecessary;
            }
        });
    }
}
/** Converts a Monaco marker to a VS Code Diagnostic. */
export function toDiagnostic(marker: monacoEditor.IMarkerData): Diagnostic {
    let diagnostic = MonacoDiagnostics.get(marker);
    if (diagnostic) return diagnostic;
    diagnostic = new MonacoDiagnostic(marker);
    MonacoDiagnostics.set(marker, diagnostic);
    return diagnostic;
}

/** Converts a VS Code Diagnostic to a Monaco marker. */
export function fromDiagnostic(diagnostic: Diagnostic): monacoEditor.IMarkerData {
    if (diagnostic instanceof MonacoDiagnostic) {
        return diagnostic.marker;
    }
    throw new Error('Cannot convert Diagnostic to Monaco marker.');
}

/** Converts a Monaco CodeAction to a VS Code CodeAction. */
export function toCodeAction(action: monacoLanguages.CodeAction): CodeAction {
    const ca = new CodeAction(action.title);
    switch (action.kind) {
        case 'quickfix':
            ca.kind = CodeActionKind.QuickFix;
            break;
        case 'refactor':
            ca.kind = CodeActionKind.Refactor;
            break;
        case 'refactor.extract':
            ca.kind = CodeActionKind.RefactorExtract;
            break;
        case 'refactor.inline':
            ca.kind = CodeActionKind.RefactorInline;
            break;
        case 'refactor.move':
            ca.kind = CodeActionKind.RefactorMove;
            break;
        case 'refactor.rewrite':
            ca.kind = CodeActionKind.RefactorRewrite;
            break;
        case 'source':
            ca.kind = CodeActionKind.Source;
            break;
        case 'source.organizeImports':
            ca.kind = CodeActionKind.SourceOrganizeImports;
            break;
        case 'source.fixAll':
            ca.kind = CodeActionKind.SourceFixAll;
            break;
        case 'notebook':
            ca.kind = CodeActionKind.Notebook;
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
    return ca;
}
