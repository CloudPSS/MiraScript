import { type CancellationToken, type editor, languages } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { DiagnosticCode } from 'mira-wasm';

/** @inheritdoc */
class DocumentSymbolProvider extends Provider implements languages.DocumentSymbolProvider {
    /** @inheritdoc */
    async provideDocumentSymbols(
        model: editor.ITextModel,
        token: CancellationToken,
    ): Promise<languages.DocumentSymbol[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const { locals } = compiled.groupedTags(model);
        return locals.map(({ definition }) => {
            let kind: languages.SymbolKind = languages.SymbolKind.Variable;
            let name: string | undefined;
            // eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
            switch (definition.code) {
                case DiagnosticCode.UnusedParameterIt:
                case DiagnosticCode.ParameterIt:
                    name = `it`;
                    break;
                case DiagnosticCode.LocalFunction:
                    kind = languages.SymbolKind.Function;
                    break;
            }
            return {
                name: name ?? model.getValueInRange(definition.range),
                detail: '',
                kind,
                range: definition.range,
                tags: [],
                selectionRange: definition.range,
                children: [],
            } satisfies languages.DocumentSymbol;
        });
    }
}
languages.registerDocumentSymbolProvider('mirascript', new DocumentSymbolProvider());
