import { type CancellationToken, type editor, languages, type Position, Range } from '@private/monaco-editor';
import { Provider } from './worker-helper';

/** @inheritdoc */
class RangeProvider extends Provider implements languages.FoldingRangeProvider, languages.SelectionRangeProvider {
    /** @inheritdoc */
    async provideFoldingRanges(
        model: editor.ITextModel,
        context: languages.FoldingContext,
        token: CancellationToken,
    ): Promise<languages.FoldingRange[] | undefined> {
        const compiled = await Provider.getCompileResult(model);
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
        const compiled = await Provider.getCompileResult(model);
        if (!compiled) return undefined;
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

const instance = new RangeProvider();
languages.registerSelectionRangeProvider('mirascript', instance);
languages.registerFoldingRangeProvider('mirascript', instance);
