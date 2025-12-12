import test, { type ThrowsExpectation } from 'ava';
import { compile, VmError } from '@mirascript/mirascript';
import { createScript } from '@mirascript/mirascript/subtle';

const compileAndRun = test.macro<[string, unknown]>({
    exec: async (t, code, expected) => {
        const expectError =
            expected &&
            typeof expected == 'object' &&
            'instanceOf' in expected &&
            typeof expected.instanceOf == 'function'
                ? (expected as ThrowsExpectation<Error>)
                : null;
        let scriptSource;
        {
            if (expectError) {
                await t.throwsAsync(async () => {
                    const script = await compile(code);
                    scriptSource = script.toString();
                    script();
                }, expectError);
            } else {
                const script = await compile(code);
                scriptSource = script.toString();
                const result = script();
                t.deepEqual(result, expected);
            }
        }
        {
            // wrap in a block to bypass fast route
            code = `{${code}}`;
            if (expectError) {
                await t.throwsAsync(async () => {
                    const script = await compile(code);
                    script();
                }, expectError);
            } else {
                const script = await compile(code);
                const result = script();
                t.deepEqual(result, expected);
            }
        }
        if (scriptSource) {
            // create from source code
            const script = createScript(code, 'Script', scriptSource);
            if (expectError) {
                t.throws(() => {
                    script();
                }, expectError);
            } else {
                const result = script();
                t.deepEqual(result, expected);
            }
        }
    },
    title: (providedTitle = 'value', code) => `${providedTitle} ${code}`,
});

test('keyword literal', compileAndRun, 'nil', null);
test('keyword literal', compileAndRun, 'nan', Number.NaN);
test('keyword literal', compileAndRun, 'inf', Number.POSITIVE_INFINITY);
test('keyword literal', compileAndRun, '+inf', Number.POSITIVE_INFINITY);
test('keyword literal', compileAndRun, '-inf', Number.NEGATIVE_INFINITY);
test('keyword literal', compileAndRun, '+ inf', Number.POSITIVE_INFINITY);
test('keyword literal', compileAndRun, '- inf', Number.NEGATIVE_INFINITY);
test('keyword literal', compileAndRun, 'true  ', true);
test('keyword literal', compileAndRun, '  false', false);

test('number literal', compileAndRun, '1', 1);
test('number literal', compileAndRun, '1.0', 1);
test('number literal', compileAndRun, '1.1', 1.1);
test('number literal', compileAndRun, '1.1e2', 110);
test('number literal', compileAndRun, '1.1e-2', 0.011);
test('number literal', compileAndRun, '1.1e+2', 110);
test('number literal', compileAndRun, '+1', 1);
test('number literal', compileAndRun, '-1', -1);
test('number literal', compileAndRun, '0', 0);
test('number literal', compileAndRun, '-0', -0);
test('number literal', compileAndRun, '-1e+2', -100);

test('string literal', compileAndRun, '`hello`', 'hello');
test('string literal with escapes', compileAndRun, '`hello\\nworld`', 'hello\nworld');
test('string literal with unicode escapes', compileAndRun, '`\\u{0041}\\u{0042}\\u{0043}`', 'ABC');
test('empty string literal', compileAndRun, '``', '');

test('empty', compileAndRun, '', null);
test('whitespace', compileAndRun, ' ', null);
test('whitespaces', compileAndRun, ' \n', null);

test('identifier', compileAndRun, 'PI', Math.PI);
test('identifier with whitespace', compileAndRun, ' \n SQRT1_2\n\r\n ', Math.SQRT1_2);

test('keyword', compileAndRun, 'if', {
    instanceOf: Error,
    message: /MissingCloseBrace/,
});
test('non-existent identifier', compileAndRun, '@nonExistent', {
    instanceOf: VmError,
    message: "Global variable '@nonExistent' is not defined.",
});
test('non-existent identifier', compileAndRun, 'no', {
    instanceOf: VmError,
    message: "Global variable 'no' is not defined.",
});
test('bad expression', compileAndRun, '++', {
    instanceOf: Error,
    message: /UnknownExpression/,
});
