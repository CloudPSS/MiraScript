import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';

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
        const decl = compiled.definition(model, position);
        if (!decl) return [];
        const links: languages.DocumentHighlight[] = decl.def.references.map((u) => ({
            kind: languages.DocumentHighlightKind.Read,
            range: u.range,
        }));
        if (decl.def.definition && !Range.isEmpty(decl.def.definition.range)) {
            links.push({
                kind: languages.DocumentHighlightKind.Write,
                range: decl.def.definition.range,
            });
        }
        return links;
    }
}
languages.registerDocumentHighlightProvider('mirascript', new DocumentHighlightProvider());
