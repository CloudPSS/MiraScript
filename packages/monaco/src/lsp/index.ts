import type { Monaco, IDisposable } from '../index.js';
import type { languages } from '@private/monaco-editor';
import '../basic/index.js';

import { CodeActionProvider } from './code-action-provider.js';
import { ColorProvider } from './color-provider.js';
import { CompletionItemProvider } from './completion-item-provider.js';
import { DefinitionReferenceProvider } from './definition-reference-provider.js';
import {} from './diagnostics.js';
import { DocumentHighlightProvider } from './document-highlight-provider.js';
import { DocumentSymbolProvider } from './document-symbol-provider.js';
import { FormatterProvider } from './formatter-provider.js';
import { HoverProvider } from './hover-provider.js';
import { InlayHintsProvider } from './inlay-hints-provider.js';
import { RangeProvider } from './range-provider.js';
import { RenameProvider } from './rename-provider.js';
import { DocumentSemanticTokensProvider } from './semantic-tokens-provider.js';
import { SignatureHelpProvider } from './signature-help-provider.js';

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

/** 注册 LSP 相关的 Monaco 编辑器功能 */
export function registerLSP(monaco: Monaco): IDisposable[] {
    const codeActionProvider = new CodeActionProvider(monaco);
    const colorProvider = new ColorProvider(monaco);
    const completionItemProvider = new CompletionItemProvider(monaco);
    const definitionReferenceProvider = new DefinitionReferenceProvider(monaco);
    const documentHighlightProvider = new DocumentHighlightProvider(monaco);
    const documentSymbolProvider = new DocumentSymbolProvider(monaco);
    const formatterProvider = new FormatterProvider(monaco);
    const hoverProvider = new HoverProvider(monaco);
    const inlayHintsProvider = new InlayHintsProvider(monaco);
    const rangeProvider = new RangeProvider(monaco);
    const renameProvider = new RenameProvider(monaco);
    const documentSemanticTokensProvider = new DocumentSemanticTokensProvider(monaco);
    const signatureHelpProvider = new SignatureHelpProvider(monaco);

    const language: languages.LanguageSelector = ['mirascript', 'mirascript-template'];
    const { languages } = monaco;
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
