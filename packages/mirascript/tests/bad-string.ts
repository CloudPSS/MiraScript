import test, { type ThrowsExpectation } from 'ava';
import { compile } from '@mirascript/mirascript';
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

test('Unterminated string literal', compileAndRun, '`hello', {
    instanceOf: Error,
    message: /UnterminatedString/,
});
test('Unterminated string literal with expr interpolation', compileAndRun, '`hello $(name', {
    instanceOf: Error,
    message: /UnterminatedInterpolation/,
});
test('Unterminated string literal with expr format interpolation', compileAndRun, '`hello $(name:', {
    instanceOf: Error,
    message: /UnterminatedString/,
});
test('Unterminated string literal with block interpolation', compileAndRun, '`hello ${name', {
    instanceOf: Error,
    message: /UnterminatedInterpolation/,
});
test('Empty expr interpolation', compileAndRun, '`hello $()!`', {
    instanceOf: Error,
    message: /EmptyInterpolation\(1:10\)/,
});
test('Empty expr interpolation with format', compileAndRun, '`hello $(:.2)!`', {
    instanceOf: Error,
    message: /EmptyInterpolation\(1:10\)/,
});
test('Bad expr interpolation with unknown token', compileAndRun, '`hello $(~)!`', {
    instanceOf: Error,
    message: /EmptyInterpolation\(1:10-11\)/,
});
test('Bad expr interpolation with unknown tokens', compileAndRun, '`hello $(~~)!`', {
    instanceOf: Error,
    message: /EmptyInterpolation\(1:10-12\)/,
});
