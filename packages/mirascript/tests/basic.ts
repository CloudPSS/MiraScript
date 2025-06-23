import test from 'ava';
import { compile } from 'mirascript';

const compileAndRun = test.macro<[string, unknown]>({
    exec: async (t, code, expected) => {
        {
            const script = await compile(code);
            const result = script();
            t.deepEqual(result, expected);
        }
        {
            // wrap in a block to bypass fast route
            const script = await compile(`{${code}}`);
            const result = script();
            t.deepEqual(result, expected);
        }
    },
    title: (providedTitle = 'value', code) => `${providedTitle} ${code}`,
});

test('keyword literal', compileAndRun, 'nil', null);
test('keyword literal', compileAndRun, 'nan', Number.NaN);
test('keyword literal', compileAndRun, 'inf', Number.POSITIVE_INFINITY);
test('keyword literal', compileAndRun, '-inf', Number.NEGATIVE_INFINITY);
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
