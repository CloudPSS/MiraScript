import { languages, type IDisposable } from '../monaco-api.js';
import '../basic/index.js';

import { CodeActionProvider } from './providers/code-action-provider.js';
import { ColorProvider } from './providers/color-provider.js';
import { CompletionItemProvider } from './providers/completion-item-provider.js';
import { DefinitionReferenceProvider } from './providers/definition-reference-provider.js';
import {} from './diagnostics.js';
import { DocumentHighlightProvider } from './providers/document-highlight-provider.js';
import { DocumentSymbolProvider } from './providers/document-symbol-provider.js';
import { FormatterProvider } from './providers/formatter-provider.js';
import { HoverProvider } from './providers/hover-provider.js';
import { InlayHintsProvider } from './providers/inlay-hints-provider.js';
import { RangeProvider } from './providers/range-provider.js';
import { RenameProvider } from './providers/rename-provider.js';
import { DocumentSemanticTokensProvider } from './providers/semantic-tokens-provider.js';
import { SignatureHelpProvider } from './providers/signature-help-provider.js';

export {
    CodeActionProvider,
    ColorProvider,
    CompletionItemProvider,
    DefinitionReferenceProvider,
    DocumentHighlightProvider,
    DocumentSymbolProvider,
    FormatterProvider,
    HoverProvider,
    InlayHintsProvider,
    RangeProvider,
    RenameProvider,
    DocumentSemanticTokensProvider,
    SignatureHelpProvider,
};

/** 注册 LSP 相关的  编辑器功能 */
export function registerLSP(): IDisposable[] {
    const codeActionProvider = new CodeActionProvider();
    const colorProvider = new ColorProvider();
    const completionItemProvider = new CompletionItemProvider();
    const definitionReferenceProvider = new DefinitionReferenceProvider();
    const documentHighlightProvider = new DocumentHighlightProvider();
    const documentSymbolProvider = new DocumentSymbolProvider();
    const formatterProvider = new FormatterProvider();
    const hoverProvider = new HoverProvider();
    const inlayHintsProvider = new InlayHintsProvider();
    const rangeProvider = new RangeProvider();
    const renameProvider = new RenameProvider();
    const documentSemanticTokensProvider = new DocumentSemanticTokensProvider();
    const signatureHelpProvider = new SignatureHelpProvider();

    const language: languages.LanguageSelector = ['mirascript', 'mirascript-template'];
    return [
        languages.registerCodeActionProvider(language, codeActionProvider),
        languages.registerColorProvider(language, colorProvider),
        languages.registerCompletionItemProvider(language, completionItemProvider),

        languages.registerDefinitionProvider(language, definitionReferenceProvider),
        languages.registerReferenceProvider(language, definitionReferenceProvider),

        languages.registerDocumentHighlightProvider(language, documentHighlightProvider),
        languages.registerDocumentSymbolProvider(language, documentSymbolProvider),

        languages.registerDocumentFormattingEditProvider(language, formatterProvider),
        languages.registerDocumentRangeFormattingEditProvider(language, formatterProvider),
        languages.registerOnTypeFormattingEditProvider(language, formatterProvider),

        languages.registerHoverProvider(language, hoverProvider),
        languages.registerInlayHintsProvider(language, inlayHintsProvider),

        languages.registerFoldingRangeProvider(language, rangeProvider),
        languages.registerSelectionRangeProvider(language, rangeProvider),

        languages.registerRenameProvider(language, renameProvider),
        languages.registerDocumentSemanticTokensProvider(language, documentSemanticTokensProvider),
        languages.registerSignatureHelpProvider(language, signatureHelpProvider),
    ];
}
