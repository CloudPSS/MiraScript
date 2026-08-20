import test from 'ava';
import { compileWithIL, emitIL, generateBytecode, type IRange } from '@mirascript/mirascript/subtle';

test('compileWithIL returns an executable script and readable IL', async (t) => {
    const { script, il } = await compileWithIL('let x = 1 + 2; x', { pretty: true });

    t.is(script(), 3);
    t.snapshot(il, 'il.txt');
});

test('IL preserves structured instruction indentation', async (t) => {
    const { il } = await compileWithIL('if true { [1, 2] } else { (value: 3) }');
    t.snapshot(il, 'il-structured.txt');
});

test('IL marks wide instructions', async (t) => {
    const source = `[${Array.from({ length: 300 }, (_, index) => index).join(',')}]`;
    const { script, il } = await compileWithIL(source);

    t.is((script() as unknown[]).length, 300);
    t.snapshot(il, 'il-wide.txt');
});

test('IL appends original source lines from source maps', async (t) => {
    const source = `fn abs(x) {
  if x < 0 { -x } else { x }
}
abs(-2)`;
    const { il } = await compileWithIL(source, {
        diagnostic_position_encoding: 'Utf32',
    });

    const commentLines = il.split('\n').filter((line) => line.includes('; '));
    const comments = commentLines.map((line) => line.slice(line.indexOf('; ') + 2));

    t.deepEqual(comments, ['fn abs(x) {', 'if x < 0 { -x } else { x }', '}', 'abs(-2)']);
    t.is(new Set(commentLines.map((line) => line.indexOf(';'))).size, 1);
});

test('long annotated instructions wrap comments without widening other padding', async (t) => {
    const source = `f(${Array.from({ length: 40 }, (_, index) => index).join(',')})`;
    const [bytecode] = await generateBytecode(source, {});
    const plainLines = emitIL(bytecode!).split('\n');
    const plainInstructions = plainLines.slice(plainLines.indexOf('.code') + 1);
    let longestInstruction = '';
    for (const line of plainInstructions) {
        if (line.length > longestInstruction.length) longestInstruction = line;
    }
    const longestIndex = plainInstructions.indexOf(longestInstruction);
    const firstLine: IRange = { startLineNumber: 1, startColumn: 1, endLineNumber: 1, endColumn: 2 };
    const secondLine: IRange = { startLineNumber: 2, startColumn: 1, endLineNumber: 2, endColumn: 2 };
    const ranges = Array.from({ length: plainInstructions.length }, () => firstLine);
    ranges[longestIndex] = secondLine;

    const lines = emitIL(bytecode!, {
        source: 'short source\nlong source',
        ranges,
    }).split('\n');
    const shortComment = lines.find((line) => line.endsWith('; short source'))!;
    const longInstructionIndex = lines.indexOf(longestInstruction);
    const wrappedComment = lines[longInstructionIndex + 1];

    t.true(longestInstruction.length >= 80);
    t.false(longestInstruction.includes(';'));
    t.true(wrappedComment.endsWith('; long source'));
    t.is(wrappedComment.indexOf(';'), shortComment.indexOf(';'));
    t.true(shortComment.indexOf(';') < longestInstruction.length);
});

test('compileWithIL reports compiler errors', async (t) => {
    await t.throwsAsync(compileWithIL('1 +'), { message: /Failed to compile/ });
});
