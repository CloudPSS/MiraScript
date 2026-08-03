import test from 'ava';
import {
    getDiagnosticMessage,
    formatDiagnostics,
    formatDiagnosticMessage,
    DiagnosticCode,
    type SourceDiagnostic,
    type SourceReference,
} from '@mirascript/mirascript/subtle';

test('getDiagnosticMessage with invalid code', (t) => {
    const invalidCodes = [-1, 0xffff, 1.5, Number.NaN, Number.POSITIVE_INFINITY, 'x', {}, '1'];
    for (const code of invalidCodes) {
        t.throws(
            () => {
                // @ts-expect-error Testing invalid codes
                getDiagnosticMessage(code);
            },
            { instanceOf: RangeError },
        );
    }
});

test('getDiagnosticMessage with valid code', (t) => {
    // @ts-expect-error Testing invalid codes
    const message = getDiagnosticMessage(0);
    t.is(message, null);
});

test('formatDiagnostics with empty diagnostics', (t) => {
    const diagnostics: SourceDiagnostic[] = [];
    const formatted = formatDiagnostics(diagnostics, '', 'test.mira');
    t.deepEqual(formatted, []);
});

test('formatDiagnostics with invalid diagnostics', (t) => {
    const diagnostic: SourceDiagnostic = {
        references: [],
        code: 0 as DiagnosticCode,
        range: { startLineNumber: 1, startColumn: 1, endLineNumber: 2, endColumn: 2 },
    };
    const formatted = formatDiagnostics([diagnostic], '', 'test.mira');
    t.deepEqual(formatted, ['  Unknown(test.mira:1:1-2:2): Unknown diagnostic code: 0']);
});
test('formatDiagnostics with valid diagnostics', (t) => {
    const diagnostic: SourceDiagnostic = {
        references: [],
        code: DiagnosticCode.UnexpectedToken,
        range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 2 },
    };
    const formatted = formatDiagnostics([diagnostic], '', 'test.mira');
    t.deepEqual(formatted, ['  UnexpectedToken(test.mira:1:1-2): 发现意外的记号']);
});

test('formatDiagnostics with ref diagnostics', (t) => {
    const diagnostic: SourceDiagnostic = {
        references: [],
        code: DiagnosticCode.DuplicateVariableDeclaration,
        range: { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 2 },
    };
    const ref: SourceReference = {
        diagnostic,
        code: DiagnosticCode.VariableDeclaredHere,
        range: { startLineNumber: 1, startColumn: 3, endLineNumber: 1, endColumn: 4 },
    };
    (diagnostic.references as SourceReference[]).push(ref);
    const formatted = formatDiagnostics([diagnostic], '', 'test.mira');
    t.deepEqual(formatted, [
        `  DuplicateVariableDeclaration(test.mira:1:1-2): 该变量已...\n    (1:3-4): ...在此处声明`,
    ]);
});

test('formatDiagnosticMessage with invalid code', (t) => {
    const message = formatDiagnosticMessage(0 as DiagnosticCode);
    t.is(message, 'Unknown diagnostic code: 0');
});

test('formatDiagnosticMessage with valid code', (t) => {
    const message = formatDiagnosticMessage(DiagnosticCode.UnexpectedToken);
    t.is(message, '发现意外的记号');
});

test('formatDiagnosticMessage with replacement', (t) => {
    const message = formatDiagnosticMessage(DiagnosticCode.InvalidReservedKeyword, 'if');
    t.is(message, '`if` 是保留关键字，不能用作标识符');
});

test('formatDiagnosticMessage with replacement function', (t) => {
    const message = formatDiagnosticMessage(DiagnosticCode.InvalidReservedKeyword, () => 'if');
    t.is(message, '`if` 是保留关键字，不能用作标识符');
});

test('formatDiagnosticMessage with replacement function returning undefined', (t) => {
    const message = formatDiagnosticMessage(DiagnosticCode.InvalidReservedKeyword, () => undefined);
    t.is(message, '`` 是保留关键字，不能用作标识符');
});

test('formatDiagnosticMessage with replacement containing $', (t) => {
    const message = formatDiagnosticMessage(DiagnosticCode.InvalidReservedKeyword, '$if$');
    t.is(message, '`$if$` 是保留关键字，不能用作标识符');
});
