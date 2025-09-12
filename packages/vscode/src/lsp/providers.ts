import {
    type CompletionItem,
    CompletionItemKind,
    CompletionItemTag,
    CompletionList,
    CompletionTriggerKind,
    Disposable,
    DocumentHighlight,
    DocumentHighlightKind,
    DocumentSymbol,
    Hover,
    InlayHint,
    InlayHintKind,
    InlayHintLabelPart,
    languages,
    ParameterInformation,
    SemanticTokens,
    SignatureHelp,
    SignatureHelpTriggerKind,
    SignatureInformation,
    SymbolKind,
    WorkspaceEdit,
} from 'vscode';
import {
    FormatterProvider,
    DocumentSemanticTokensProvider,
    DocumentHighlightProvider,
    DocumentSymbolProvider,
    HoverProvider,
    RenameProvider,
    InlayHintsProvider,
    CompletionItemProvider,
    SignatureHelpProvider,
} from '@mirascript/monaco/lsp';
import { ModelAdapter } from '../adapter/model.js';
import {
    CompletionItemInsertTextRule,
    fromPosition,
    fromRange,
    toCompletionItem,
    toMarkdownString,
    toPosition,
    toRange,
    toTextEdit,
} from '../adapter/utils.js';
import { registerMonacoApi } from '@mirascript/monaco';
import * as monaco from '@private/monaco-editor/baseapi';
import type { languages as monacoLanguages } from '@private/monaco-editor';

/**
 * Manages all language service providers.
 */
export class ProvidersManager extends Disposable {
    private readonly disposables: Disposable[] = [];

    constructor() {
        super(() => {
            for (const disposable of this.disposables) {
                disposable.dispose();
            }
        });
        const api = { ...monaco };
        (api as Record<string, unknown>)['languages'] = {
            SymbolKind,
            DocumentHighlightKind,
            InlayHintKind,
            CompletionItemKind,
            CompletionTriggerKind,
            CompletionItemTag,
            CompletionItemInsertTextRule,
            SignatureHelpTriggerKind,
        };
        registerMonacoApi(api);
        this.registerProviders();
    }

    /** 注册 Providers */
    private registerProviders(): void {
        const selector = ['mirascript', 'mirascript-template'];

        // const codeActionProvider = new CodeActionProvider();
        // const colorProvider = new ColorProvider();

        // const definitionReferenceProvider = new DefinitionReferenceProvider();

        const documentHighlightProvider = new DocumentHighlightProvider();
        const documentSymbolProvider = new DocumentSymbolProvider();

        const formatterProvider = new FormatterProvider();

        const inlayHintsProvider = new InlayHintsProvider();
        const hoverProvider = new HoverProvider();

        // const rangeProvider = new RangeProvider();

        const documentSemanticTokensProvider = new DocumentSemanticTokensProvider();
        const renameProvider = new RenameProvider();
        const completionItemProvider = new CompletionItemProvider();
        const signatureHelpProvider = new SignatureHelpProvider();

        this.disposables.push(
            languages.registerDocumentFormattingEditProvider(selector, {
                provideDocumentFormattingEdits: async (document, options, token) => {
                    const result = await formatterProvider.provideDocumentFormattingEdits(
                        new ModelAdapter(document),
                        options,
                        token,
                    );
                    if (!result) return result;
                    return result.map(toTextEdit);
                },
            }),
            languages.registerDocumentSemanticTokensProvider(
                selector,
                {
                    provideDocumentSemanticTokens: async (document, token) => {
                        const result = await documentSemanticTokensProvider.provideDocumentSemanticTokens(
                            new ModelAdapter(document),
                            null,
                            token,
                        );
                        if (!result) return null;
                        return new SemanticTokens(result.data);
                    },
                },
                {
                    tokenTypes: [
                        'variable',
                        'variable',
                        'variable',
                        'variable',
                        'function',
                        'namespace',
                        'property',
                        'keyword',
                        'parameter',
                        'parameter',
                    ],
                    tokenModifiers: ['readonly', '', 'readonly', '', '', 'readonly', '', 'controlFlow', 'readonly', ''],
                },
            ),
            languages.registerHoverProvider(selector, {
                provideHover: async (document, position, token) => {
                    const result = await hoverProvider.provideHover(
                        new ModelAdapter(document),
                        fromPosition(position),
                        token,
                    );
                    if (!result) return null;
                    return new Hover(result.contents.map(toMarkdownString), result.range && toRange(result.range));
                },
            }),
            languages.registerRenameProvider(selector, {
                prepareRename: async (document, position, token) => {
                    const result = await renameProvider.resolveRenameLocation(
                        new ModelAdapter(document),
                        fromPosition(position),
                        token,
                    );
                    if (!result) return null;
                    if ('rejectReason' in result) {
                        throw new Error(result.rejectReason);
                    }
                    return {
                        range: toRange(result.range),
                        placeholder: result.text,
                    };
                },
                provideRenameEdits: async (document, position, newName, token) => {
                    const result = await renameProvider.provideRenameEdits(
                        new ModelAdapter(document),
                        fromPosition(position),
                        newName,
                        token,
                    );
                    if (!result) return null;
                    if ('rejectReason' in result) {
                        throw new Error(result.rejectReason);
                    }
                    const edits = new WorkspaceEdit();
                    for (const edit of result.edits) {
                        if ('textEdit' in edit) {
                            edits.replace(
                                document.uri,
                                toRange(edit.textEdit.range),
                                edit.textEdit.text,
                                edit.metadata,
                            );
                            continue;
                        }
                    }
                    return edits;
                },
            }),
            languages.registerDocumentHighlightProvider(selector, {
                provideDocumentHighlights: async (document, position, token) => {
                    const result = await documentHighlightProvider.provideDocumentHighlights(
                        new ModelAdapter(document),
                        fromPosition(position),
                        token,
                    );
                    if (!result) return null;
                    return result.map((item) => new DocumentHighlight(toRange(item.range), item.kind));
                },
            }),
            languages.registerDocumentSymbolProvider(selector, {
                provideDocumentSymbols: async (document, token) => {
                    const result = await documentSymbolProvider.provideDocumentSymbols(
                        new ModelAdapter(document),
                        token,
                    );
                    if (!result) return null;
                    const toDocumentSymbol = (item: monacoLanguages.DocumentSymbol): DocumentSymbol => {
                        const r = new DocumentSymbol(
                            item.name,
                            item.detail,
                            item.kind,
                            toRange(item.range),
                            toRange(item.selectionRange),
                        );
                        if (item.children?.length) {
                            r.children = item.children.map(toDocumentSymbol);
                        }
                        return r;
                    };
                    return result.map(toDocumentSymbol);
                },
            }),
            languages.registerInlayHintsProvider(selector, {
                provideInlayHints: async (document, range, token) => {
                    const result = await inlayHintsProvider.provideInlayHints(
                        new ModelAdapter(document),
                        fromRange(range),
                        token,
                    );
                    if (!result) return null;
                    return result.hints.map((item) => {
                        const label =
                            typeof item.label === 'string'
                                ? item.label
                                : item.label.map((part) => {
                                      const p = new InlayHintLabelPart(part.label);
                                      p.tooltip = toMarkdownString(part.tooltip);
                                      return p;
                                  });
                        const h = new InlayHint(toPosition(item.position), label, item.kind);
                        h.paddingLeft = item.paddingLeft;
                        h.paddingRight = item.paddingRight;
                        h.tooltip = toMarkdownString(item.tooltip);
                        h.textEdits = item.textEdits?.map(toTextEdit);
                        return h;
                    });
                },
            }),
            languages.registerCompletionItemProvider(
                selector,
                {
                    provideCompletionItems: async (document, position, token, context) => {
                        const result = await completionItemProvider.provideCompletionItems(
                            new ModelAdapter(document),
                            fromPosition(position),
                            context,
                            token,
                        );
                        if (!result) return null;
                        return new CompletionList(result.suggestions.map(toCompletionItem), result.incomplete);
                    },
                    resolveCompletionItem: (item, token) => {
                        const i = item as CompletionItem & { original: never };
                        const result = completionItemProvider.resolveCompletionItem(i.original, token);
                        return result ? toCompletionItem(result) : i;
                    },
                },
                ...completionItemProvider.triggerCharacters,
            ),
            languages.registerSignatureHelpProvider(
                selector,
                {
                    provideSignatureHelp: async (document, position, token, context) => {
                        const result = await signatureHelpProvider.provideSignatureHelp(
                            new ModelAdapter(document),
                            fromPosition(position),
                            token,
                            context,
                        );
                        if (!result) return null;
                        const s = new SignatureHelp();
                        s.signatures = result.value.signatures.map((sig) => {
                            const i = new SignatureInformation(sig.label, toMarkdownString(sig.documentation));
                            i.parameters = sig.parameters?.map(
                                (param) => new ParameterInformation(param.label, toMarkdownString(param.documentation)),
                            );
                            i.activeParameter = sig.activeParameter;
                            return i;
                        });
                        s.activeParameter = result.value.activeParameter;
                        s.activeSignature = result.value.activeSignature;
                        return s;
                    },
                },
                ...signatureHelpProvider.signatureHelpTriggerCharacters,
                ...signatureHelpProvider.signatureHelpRetriggerCharacters,
            ),
        );
    }
}
