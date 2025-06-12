import type { CancellationToken, editor, languages, Position } from 'monaco-editor';
import { Provider } from './base.js';

/** @inheritdoc */
export class RangeProvider
    extends Provider
    implements languages.FoldingRangeProvider, languages.SelectionRangeProvider
{
    /** @inheritdoc */
    async provideFoldingRanges(
        model: editor.ITextModel,
        context: languages.FoldingContext,
        token: CancellationToken,
    ): Promise<languages.FoldingRange[] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        // 暂时没必要
        return undefined;
    }
    /** @inheritdoc */
    async provideSelectionRanges(
        model: editor.ITextModel,
        positions: Position[],
        token: CancellationToken,
    ): Promise<languages.SelectionRange[][] | undefined> {
        const compiled = await this.getCompileResult(model);
        if (!compiled) return undefined;
        const { Range } = this.monaco;
        const { ranges } = compiled.groupedTags(model);
        return positions.map((pos) => {
            return ranges
                .filter((r) => Range.containsPosition(r.range, pos))
                .map((t) => ({
                    range: t.range,
                }));
        });
    }
}
