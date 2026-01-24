import { type editor, Range, type IPosition, type IRange } from '../monaco-api.js';

/** 检查位置是否在范围内，且范围非空 */
export function strictContainsPosition(range: IRange, position: IPosition): boolean {
    return !Range.isEmpty(range) && Range.containsPosition(range, position);
}

/** 获取单词 */
export function wordAt(model: editor.ITextModel, position: IPosition): { word: string; range: Range } | undefined {
    const word = model.getWordAtPosition(position);
    if (!word) return undefined;
    const range = new Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn);
    return { word: word.word, range };
}
