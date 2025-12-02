import test, { type ThrowsExpectation } from 'ava';
import { compile, VmError } from '@mirascript/mirascript';

const compileAndRun = test.macro<[string, unknown]>({
    exec: async (t, code, expected) => {
        const expectError =
            expected &&
            typeof expected == 'object' &&
            'instanceOf' in expected &&
            typeof expected.instanceOf == 'function'
                ? (expected as ThrowsExpectation<Error>)
                : null;
        {
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
test('keyword literal', compileAndRun, 'true', true);
test('keyword literal', compileAndRun, 'false', false);

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

test('empty', compileAndRun, '', null);
test('whitespace', compileAndRun, ' ', null);
test('whitespaces', compileAndRun, ' \n', null);

test('identifier', compileAndRun, 'PI', Math.PI);
test('identifier with whitespace', compileAndRun, ' PI\n\r\n ', Math.PI);

test('non-existent identifier', compileAndRun, '@nonExistent', {
    instanceOf: VmError,
    message: "Global variable '@nonExistent' is not defined.",
});
test('bad expression', compileAndRun, '++', {
    instanceOf: Error,
    message: /UnknownExpression/,
});
