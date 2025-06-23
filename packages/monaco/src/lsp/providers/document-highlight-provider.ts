import { DiagnosticCode } from 'mirascript';
import { languages, type editor, type Position, type CancellationToken, Range } from 'monaco-editor';
import { keywords } from '../../constants.js';
import { Provider } from './base.js';

/** @inheritdoc */
export class DocumentHighlightProvider extends Provider implements languages.DocumentHighlightProvider {
    /** @inheritdoc */
    async provideDocumentHighlights(
        model: editor.ITextModel,
        position: Position,
        token: CancellationToken,
    ): Promise<languages.DocumentHighlight[] | undefined> {
        const word = model.getWordAtPosition(position);
        if (word && keywords().includes(word.word)) {
            const result = await this.highlightKeyword(model, position, word.word);
            if (result) return result;
        }
        return this.getDocumentHighlightsFromDefinition(model, position);
    }

    /** 从 definition 生成 DocumentHighlight 列表 */
    private async getDocumentHighlightsFromDefinition(
        model: editor.ITextModel,
        position: Position,
    ): Promise<languages.DocumentHighlight[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const def = compiled.definition(model, position)?.def;
        if (!def) return undefined;
        const links: languages.DocumentHighlight[] = def.references.map((u) => {
            const { code, range } = u;
            let kind = languages.DocumentHighlightKind.Read;
            if (
                code === DiagnosticCode.WriteLocal ||
                code === DiagnosticCode.ReadWriteLocal ||
                code === DiagnosticCode.RedeclareLocal
            ) {
                kind = languages.DocumentHighlightKind.Write;
            }
            return {
                kind,
                range,
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

    /** 高亮关键字 */
    private async highlightKeyword(
        model: editor.ITextModel,
        position: Position,
        word: string,
    ): Promise<languages.DocumentHighlight[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const kw = compiled.tagsReferences.find((kw) => Range.containsPosition(kw.range, position));
        if (!kw) return undefined;

        return kw.diagnostic.references.map((r) => ({
            kind: languages.DocumentHighlightKind.Text,
            range: r.range,
        }));
    }
}
