import test from 'ava';
import { compile } from 'mirascript';

test('syntax error', async (t) => {
    await t.throwsAsync(compile('1+'), { message: /Failed to compile/ });
});

test('compile error', async (t) => {
    await t.throwsAsync(compile('a = 12; let a = 1;'), { message: /Failed to compile/ });
});

test('buffer', async (t) => {
    const s = await compile(Buffer.from('1'));
    t.is(s.source, '<buffer>');
    t.is(s(), 1);
});

test('source map', async (t) => {
    const s = await compile('1', { sourceMap: true });
    t.is(s.source, '1');
    t.regex(s.toString(), /try/);
    t.notRegex(s.toString(), /^\s{2}/m);

    const s2 = await compile('1', { sourceMap: false });
    t.is(s2.source, '1');
    t.notRegex(s2.toString(), /try/);
});

test('pretty', async (t) => {
    const s = await compile('1', { pretty: true, sourceMap: true });
    t.is(s.source, '1');
    t.regex(s.toString(), /^\s{2}/m);
});

test('invalid number', async (t) => {
    await t.throwsAsync(compile('1.2e999'), { message: /Failed to compile/ });
    await t.throwsAsync(compile('1.2e999', { sourceMap: true }), { message: /Failed to compile/ });
});
