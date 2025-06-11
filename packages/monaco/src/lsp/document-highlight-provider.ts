import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper.js';
import { DiagnosticCode } from '@mirascript/wasm';

/** @inheritdoc */
class DocumentHighlightProvider extends Provider implements languages.DocumentHighlightProvider {
    /** @inheritdoc */
    async provideDocumentHighlights(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<languages.DocumentHighlight[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
        const def = compiled.definition(model, position)?.def;
        if (!def) return undefined;
        const links: languages.DocumentHighlight[] = def.references.map((u) => {
            let kind = languages.DocumentHighlightKind.Read;
            if (
                u.code === DiagnosticCode.WriteLocal ||
                u.code === DiagnosticCode.ReadWriteLocal ||
                u.code === DiagnosticCode.RedeclareLocal
            ) {
                kind = languages.DocumentHighlightKind.Write;
            }
            return {
                kind,
                range: u.range,
            };
        });
        if ('definition' in def && !Range.isEmpty(def.definition.range)) {
            links.push({
                kind: languages.DocumentHighlightKind.Write,
                range: def.definition.range,
            });
        }
        return links;
    }
}
languages.registerDocumentHighlightProvider('mirascript', new DocumentHighlightProvider());
