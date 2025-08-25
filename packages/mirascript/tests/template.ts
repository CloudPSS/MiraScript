import test from 'ava';
import { compile } from '@mirascript/mirascript';

const compileAndRun = test.macro<[string, unknown]>({
    exec: async (t, code, expected) => {
        {
            const script = await compile(code, { input_mode: 'Template', sourceMap: true });
            const result = script();
            t.deepEqual(result, expected);
        }
        {
            const script = await compile(code, { input_mode: 'Template', sourceMap: false });
            const result = script();
            t.deepEqual(result, expected);
        }
    },
    title: (providedTitle = '', code) => providedTitle || code,
});

test('empty', compileAndRun, '', '');
test('whitespaces', compileAndRun, ' \n', ' \n');
test('large no interpolation', compileAndRun, 'Hello, World!'.repeat(1000), 'Hello, World!'.repeat(1000));

test('interpolation', compileAndRun, 'Hello, $("World")!', 'Hello, World!');
