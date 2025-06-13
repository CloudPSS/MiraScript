import { languages, Range, type CancellationToken, type editor, type Position } from '../../monaco-api.js';
import { Provider } from './base.js';
import { DiagnosticCode } from '@mirascript/wasm';

/** @inheritdoc */
export class DocumentHighlightProvider extends Provider implements languages.DocumentHighlightProvider {
    /** @inheritdoc */
    async provideDocumentHighlights(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<languages.DocumentHighlight[] | undefined> {
        const compiled = await this.getCompileResult(model);
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
