import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';
import { strictInRange } from './utils';

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
        const decl = compiled
            .definitions(model)
            .find(
                ({ definition, references }) =>
                    (definition && strictInRange(definition.range, position)) ||
                    references.some((u) => strictInRange(u.range, position)),
            );
        if (!decl) return [];
        const links: languages.DocumentHighlight[] = decl.references.map((u) => ({
            kind: languages.DocumentHighlightKind.Read,
            range: u.range,
        }));
        if (decl.definition && !Range.isEmpty(decl.definition.range)) {
            links.push({
                kind: languages.DocumentHighlightKind.Write,
                range: decl.definition.range,
            });
        }
        return links;
    }
}
languages.registerDocumentHighlightProvider('mirascript', new DocumentHighlightProvider());
