import type { editor, IDisposable, IPosition, IRange, Position, Range, Selection, Uri } from '@private/monaco-editor';
import { EndOfLine, type TextDocument } from 'vscode';
import { fromPosition, fromRange, toPosition, toRange } from './utils.js';
import { MIRA_CONTENT_PROVIDER, MIRA_TEXT_SCHEME } from './text-provider.js';

/** empty IDisposable */
const EMPTY_DISPOSABLE: IDisposable = Object.freeze({
    dispose: () => {
        // NOOP
    },
});

const cache = new WeakMap<TextDocument, ModelAdapter>();
/**
 * Adapts a VS Code TextDocument to a Monaco Editor ITextModel.
 */
export class ModelAdapter implements editor.ITextModel {
    /** Get or create a ModelAdapter for a TextDocument */
    static from(document: TextDocument): ModelAdapter {
        let adapter = cache.get(document);
        if (!adapter) {
            adapter = new ModelAdapter(document);
            cache.set(document, adapter);
        }
        return adapter;
    }
    private constructor(readonly document: TextDocument) {}
    /** @inheritdoc */
    getCustomLineHeightsDecorations(ownerId?: number): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    undo(): void | Promise<void> {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    canUndo(): boolean {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    redo(): void | Promise<void> {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    canRedo(): boolean {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    get uri(): Uri {
        return this.document.uri;
    }
    /** @inheritdoc */
    get id(): string {
        return this.document.uri.toString();
    }
    /** @inheritdoc */
    findMatches(
        searchString: unknown,
        searchScope: unknown,
        isRegex: unknown,
        matchCase: unknown,
        wordSeparators: unknown,
        captureMatches: unknown,
        limitResultCount?: unknown,
    ): editor.FindMatch[] {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    findNextMatch(
        searchString: string,
        searchStart: IPosition,
        isRegex: boolean,
        matchCase: boolean,
        wordSeparators: string | null,
        captureMatches: boolean,
    ): editor.FindMatch | null {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    findPreviousMatch(
        searchString: string,
        searchStart: IPosition,
        isRegex: boolean,
        matchCase: boolean,
        wordSeparators: string | null,
        captureMatches: boolean,
    ): editor.FindMatch | null {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    deltaDecorations(
        oldDecorations: string[],
        newDecorations: editor.IModelDeltaDecoration[],
        ownerId?: number,
    ): string[] {
        return [];
    }
    /** @inheritdoc */
    getDecorationOptions(id: string): editor.IModelDecorationOptions | null {
        return null;
    }
    /** @inheritdoc */
    getDecorationRange(id: string): Range | null {
        return null;
    }
    /** @inheritdoc */
    getLineDecorations(lineNumber: number, ownerId?: number, filterOutValidation?: boolean): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getLinesDecorations(
        startLineNumber: number,
        endLineNumber: number,
        ownerId?: number,
        filterOutValidation?: boolean,
    ): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getDecorationsInRange(
        range: IRange,
        ownerId?: number,
        filterOutValidation?: boolean,
        onlyMinimapDecorations?: boolean,
        onlyMarginDecorations?: boolean,
    ): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getAllDecorations(ownerId?: number, filterOutValidation?: boolean): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getAllMarginDecorations(ownerId?: number): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getOverviewRulerDecorations(ownerId?: number, filterOutValidation?: boolean): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    getInjectedTextDecorations(ownerId?: number): editor.IModelDecoration[] {
        return [];
    }
    /** @inheritdoc */
    normalizeIndentation(str: string): string {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    updateOptions(newOpts: editor.ITextModelUpdateOptions): void {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    detectIndentation(defaultInsertSpaces: boolean, defaultTabSize: number): void {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    pushStackElement(): void {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    popStackElement(): void {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    pushEditOperations(
        beforeCursorState: Selection[] | null,
        editOperations: editor.IIdentifiedSingleEditOperation[],
        cursorStateComputer: editor.ICursorStateComputer,
    ): Selection[] | null {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    pushEOL(eol: editor.EndOfLineSequence): void {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    applyEdits(operations: unknown, computeUndoEdits?: unknown): editor.IValidEditOperation[] {
        throw new Error('Method not implemented.');
    }
    /** @inheritdoc */
    onDidChangeContent(listener: (e: editor.IModelContentChangedEvent) => void): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onDidChangeDecorations(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onDidChangeOptions(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onDidChangeLanguage(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onDidChangeLanguageConfiguration(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onDidChangeAttached(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    onWillDispose(): IDisposable {
        return EMPTY_DISPOSABLE;
    }
    /** @inheritdoc */
    dispose(): void {
        return;
    }

    /** @inheritdoc */
    getOptions(): editor.TextModelResolvedOptions {
        return {
            _textModelResolvedOptionsBrand: undefined,
            tabSize: 2,
            indentSize: 2,
            insertSpaces: true,
            defaultEOL: 1 satisfies editor.DefaultEndOfLine.LF,
            trimAutoWhitespace: true,
            bracketPairColorizationOptions: { enabled: true, independentColorPoolPerBracketType: true },
            originalIndentSize: 'tabSize',
        };
    }
    /** @inheritdoc */
    getVersionId(): number {
        return this.document.version;
    }
    /** @inheritdoc */
    getAlternativeVersionId(): number {
        return this.document.version;
    }
    /** @inheritdoc */
    setValue(newValue: string | editor.ITextSnapshot): void {
        // Not supported, TextDocument is readonly
        if (this.document.uri.scheme !== MIRA_TEXT_SCHEME) {
            throw new Error('setValue is not supported on ModelAdapter.');
        }
        MIRA_CONTENT_PROVIDER.setContent(
            this.document.uri,
            typeof newValue === 'string' ? newValue : (newValue.read() ?? ''),
        );
    }
    /** @inheritdoc */
    getValue(eol?: editor.EndOfLinePreference, preserveBOM?: boolean): string {
        return this.document.getText();
    }
    /** @inheritdoc */
    createSnapshot(preserveBOM?: boolean): editor.ITextSnapshot {
        const value = this.getValue();
        let done = false;
        return {
            read: () => {
                if (done) return null;
                done = true;
                return value;
            },
        };
    }
    /** @inheritdoc */
    getValueLength(eol?: editor.EndOfLinePreference, preserveBOM?: boolean): number {
        return this.getValue().length;
    }
    /** @inheritdoc */
    getValueInRange(range: IRange, eol?: editor.EndOfLinePreference): string {
        const r = { ...range };
        if (r.startColumn < 1) r.startColumn = 1;
        if (r.endColumn < 1) r.endColumn = 1;
        if (r.startLineNumber < 1) r.startLineNumber = 1;
        if (r.endLineNumber < 1) r.endLineNumber = 1;
        return this.document.getText(toRange(r));
    }
    /** @inheritdoc */
    getValueLengthInRange(range: IRange, eol?: editor.EndOfLinePreference): number {
        return this.getValueInRange(range, eol).length;
    }
    /** @inheritdoc */
    getCharacterCountInRange(range: IRange, eol?: editor.EndOfLinePreference): number {
        return [...this.getValueInRange(range, eol)].length;
    }
    /** @inheritdoc */
    getLineCount(): number {
        return this.document.lineCount;
    }
    /** @inheritdoc */
    getLineContent(lineNumber: number): string {
        return this.document.lineAt(lineNumber - 1).text;
    }
    /** @inheritdoc */
    getLineLength(lineNumber: number): number {
        return this.getLineContent(lineNumber).length;
    }
    /** @inheritdoc */
    getLinesContent(): string[] {
        const lines = [];
        const lineCount = this.getLineCount();
        for (let i = 1; i <= lineCount; i++) {
            lines.push(this.getLineContent(i));
        }
        return lines;
    }
    /** @inheritdoc */
    getEOL(): string {
        return this.document.eol === EndOfLine.LF ? '\n' : '\r\n';
    }
    /** @inheritdoc */
    getEndOfLineSequence(): editor.EndOfLineSequence {
        return this.document.eol === EndOfLine.LF
            ? (0 satisfies editor.EndOfLineSequence.LF)
            : (1 satisfies editor.EndOfLineSequence.CRLF);
    }
    /** @inheritdoc */
    getLineMinColumn(lineNumber: number): number {
        if (lineNumber < 1 || lineNumber > this.getLineCount()) return 0;
        return 1;
    }
    /** @inheritdoc */
    getLineMaxColumn(lineNumber: number): number {
        if (lineNumber < 1 || lineNumber > this.getLineCount()) return 0;
        return this.getLineLength(lineNumber) + 1;
    }
    /** @inheritdoc */
    getLineFirstNonWhitespaceColumn(lineNumber: number): number {
        if (lineNumber < 1 || lineNumber > this.getLineCount()) return 0;
        const line = this.getLineContent(lineNumber);
        const match = /\S/.exec(line);
        return match ? match.index + 1 : 0;
    }
    /** @inheritdoc */
    getLineLastNonWhitespaceColumn(lineNumber: number): number {
        if (lineNumber < 1 || lineNumber > this.getLineCount()) return 0;
        const line = this.getLineContent(lineNumber);
        const match = /\s*$/.exec(line);
        return match ? match.index : 0;
    }
    /** @inheritdoc */
    validatePosition(position: IPosition): Position {
        return fromPosition(this.document.validatePosition(toPosition(position)));
    }
    /** @inheritdoc */
    modifyPosition(position: IPosition, offset: number): Position {
        const absOffset = this.getOffsetAt(position) + offset;
        return this.getPositionAt(absOffset);
    }
    /** @inheritdoc */
    isValidRange(range: IRange): boolean {
        return this.validateRange(range).isEmpty() === false;
    }
    /** @inheritdoc */
    validateRange(range: IRange): Range {
        return fromRange(this.document.validateRange(toRange(range)));
    }
    /** @inheritdoc */
    getOffsetAt(position: IPosition): number {
        const p = toPosition(position);
        return this.document.offsetAt(p);
    }
    /** @inheritdoc */
    getPositionAt(offset: number): Position {
        const pos = this.document.positionAt(offset);
        return fromPosition(pos);
    }

    /** @inheritdoc */
    getFullModelRange(): Range {
        return this.validateRange({
            startLineNumber: 1,
            startColumn: 1,
            endLineNumber: this.getLineCount(),
            endColumn: this.getLineMaxColumn(this.getLineCount()),
        });
    }
    /** @inheritdoc */
    isDisposed(): boolean {
        return this.document.isClosed;
    }
    /** @inheritdoc */
    getLanguageId(): string {
        return this.document.languageId;
    }
    /** @inheritdoc */
    getWordAtPosition(position: IPosition): editor.IWordAtPosition | null {
        const w = this.document.getWordRangeAtPosition(toPosition(position));
        if (!w) return null;
        return {
            word: this.document.getText(w),
            startColumn: w.start.character + 1,
            endColumn: w.end.character + 1,
        };
    }
    /** @inheritdoc */
    getWordUntilPosition(position: IPosition): editor.IWordAtPosition {
        const maxCol = this.getLineMaxColumn(position.lineNumber);
        let curCol = position.column;
        while (curCol <= maxCol) {
            const word = this.getWordAtPosition({ lineNumber: position.lineNumber, column: curCol });
            if (word) return word;
            curCol++;
        }
        throw new Error('Unreachable');
    }
    /** @inheritdoc */
    setEOL(eol: editor.EndOfLineSequence): void {
        // Not supported, TextDocument is readonly
        throw new Error('setEOL is not supported on ModelAdapter.');
    }
    /** @inheritdoc */
    isAttachedToEditor(): boolean {
        return true;
    }
}
