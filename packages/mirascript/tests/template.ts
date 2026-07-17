import test from 'ava';
import { compile, compileSync } from '@mirascript/mirascript';
import { createScript } from '@mirascript/mirascript/subtle';

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
        {
            const script = compileSync(code, { input_mode: 'Template', sourceMap: true });
            const result = script();
            t.deepEqual(result, expected);
        }
        {
            const script = compileSync(code, { input_mode: 'Template', sourceMap: false });
            const result = script();
            t.deepEqual(result, expected);
        }
        {
            const script = createScript(
                code,
                'Template',
                compileSync(code, { input_mode: 'Template', sourceMap: false }).toString(),
            );
            const result = script();
            t.deepEqual(result, expected);
        }
    },
    title: (providedTitle = '', code) => providedTitle || code,
});

test('empty', compileAndRun, '', '');
test('whitespaces', compileAndRun, ' \n', ' \n');
test('special chars', compileAndRun, '\0\r\n\b\v\f\u2028\u2029<>&', '\0\r\n\b\v\f\u2028\u2029<>&');
test('large no interpolation', compileAndRun, 'Hello, World!'.repeat(1000), 'Hello, World!'.repeat(1000));
test('only one interpolation', compileAndRun, '$("Hello, World!")', 'Hello, World!');
test('interpolation', compileAndRun, 'Hello, $("World")!', 'Hello, World!');
test(
    'interpolations',
    compileAndRun,
    'Hello, $("World")! ${ let pi = PI; pi / E }',
    `Hello, World! ${(Math.PI / Math.E).toPrecision(6)}`,
);
