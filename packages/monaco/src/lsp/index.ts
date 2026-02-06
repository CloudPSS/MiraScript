import { languages, type IDisposable } from '../monaco-api.js';
import '../basic/index.js';
import type { VmContextProvider } from '../index.js';

import { Provider } from './providers/base.js';
import { CodeActionProvider } from './providers/code-action-provider.js';
import { CodeLensProvider } from './providers/code-lens-provider.js';
import { ColorProvider } from './providers/color-provider.js';
import { CompletionItemProvider } from './providers/completion-item-provider.js';
import { DefinitionReferenceProvider } from './providers/definition-reference-provider.js';
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
    CodeLensProvider,
    ColorProvider,
    CompletionItemProvider,
    DefinitionReferenceProvider,
    DocumentHighlightProvider,
    DocumentSemanticTokensProvider,
    DocumentSymbolProvider,
    FormatterProvider,
    HoverProvider,
    InlayHintsProvider,
    RangeProvider,
    RenameProvider,
    SignatureHelpProvider,
    Provider,
};

/** 支持的功能 */
export type LspFeatures = {
    codeAction: CodeActionProvider;
    codeLens: CodeLensProvider;
    color: ColorProvider;
    completionItem: CompletionItemProvider;
    definitionReference: DefinitionReferenceProvider;
    documentHighlight: DocumentHighlightProvider;
    documentSymbol: DocumentSymbolProvider;
    formatter: FormatterProvider;
    hover: HoverProvider;
    inlayHints: InlayHintsProvider;
    range: RangeProvider;
    rename: RenameProvider;
    documentSemanticTokens: DocumentSemanticTokensProvider;
    signatureHelp: SignatureHelpProvider;
};

/** 配置启用功能 */
export type LspFeaturesConfig = { [K in keyof LspFeatures]?: boolean | undefined };

/** 注册 LSP 相关的编辑器功能 */
export async function registerLSP(
    contextProvider: VmContextProvider | undefined,
    features: LspFeaturesConfig = {},
): Promise<IDisposable[]> {
    Provider.setContextProvider(contextProvider);
    const { loadModule } = await import('@mirascript/bindings/wasm');
    await loadModule();
    const disposables: IDisposable[] = [];
    const registerIfEnabled = <T extends keyof LspFeatures>(
        feature: T,
        provider: new (enabled: () => boolean) => LspFeatures[T],
        register: (language: languages.LanguageSelector, provider: LspFeatures[T]) => IDisposable | IDisposable[],
    ): void => {
        const providerInstance = new provider(() => features[feature] !== false);
        const d = register(['mirascript', 'mirascript-template'], providerInstance);
        if (Array.isArray(d)) disposables.push(...d);
        else disposables.push(d);
    };

    registerIfEnabled('codeAction', CodeActionProvider, languages.registerCodeActionProvider);
    registerIfEnabled('codeLens', CodeLensProvider, languages.registerCodeLensProvider);
    registerIfEnabled('color', ColorProvider, languages.registerColorProvider);

    registerIfEnabled('definitionReference', DefinitionReferenceProvider, (language, provider) => {
        return [
            languages.registerDefinitionProvider(language, provider),
            languages.registerReferenceProvider(language, provider),
        ];
    });

    registerIfEnabled('documentHighlight', DocumentHighlightProvider, languages.registerDocumentHighlightProvider);
    registerIfEnabled('documentSymbol', DocumentSymbolProvider, languages.registerDocumentSymbolProvider);

    registerIfEnabled('formatter', FormatterProvider, (language, provider) => {
        return [
            languages.registerDocumentFormattingEditProvider(language, provider),
            // languages.registerDocumentRangeFormattingEditProvider(language, provider),
            // languages.registerOnTypeFormattingEditProvider(language, provider),
        ];
    });

    registerIfEnabled('hover', HoverProvider, languages.registerHoverProvider);
    registerIfEnabled('inlayHints', InlayHintsProvider, languages.registerInlayHintsProvider);

    registerIfEnabled('range', RangeProvider, (language, provider) => {
        return [
            languages.registerFoldingRangeProvider(language, provider),
            languages.registerSelectionRangeProvider(language, provider),
        ];
    });

    registerIfEnabled(
        'documentSemanticTokens',
        DocumentSemanticTokensProvider,
        languages.registerDocumentSemanticTokensProvider,
    );
    registerIfEnabled('rename', RenameProvider, languages.registerRenameProvider);
    registerIfEnabled('completionItem', CompletionItemProvider, languages.registerCompletionItemProvider);
    registerIfEnabled('signatureHelp', SignatureHelpProvider, languages.registerSignatureHelpProvider);

    return disposables;
}
