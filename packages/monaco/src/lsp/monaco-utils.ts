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

/** 生成范围 */
export function rangeAt(
    ...args:
        | [lineAndColumn: { lineNumber: number; startColumn: number; endColumn: number }, empty?: null | undefined]
        | [line: { lineNumber: number }, column: { startColumn: number; endColumn: number }]
): Range {
    if (args.length === 1 || args[1] == null) {
        const { lineNumber, startColumn, endColumn } = args[0];
        return new Range(lineNumber, startColumn, lineNumber, endColumn);
    } else {
        const { lineNumber } = args[0];
        const { startColumn, endColumn } = args[1];
        return new Range(lineNumber, startColumn, lineNumber, endColumn);
    }
}
