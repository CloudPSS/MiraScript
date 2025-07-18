import test from 'ava';
import { compile } from 'mirascript';

test('syntax error', async (t) => {
    await t.throwsAsync(compile('1+'), { message: /Failed to compile/ });
});

test('compile error', async (t) => {
    await t.throwsAsync(compile('a = 12; let a = 1;'), { message: /Failed to compile/ });
});
