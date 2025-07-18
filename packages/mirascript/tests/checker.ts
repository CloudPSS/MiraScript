import test from 'ava';
import { compile, isVmScript } from 'mirascript';
import { lib } from 'mirascript/subtle';

test('isVmScript', async (t) => {
    t.true(isVmScript(await compile('nil')));
    t.true(isVmScript(await compile('1 + 1')));
    t.false(isVmScript(() => 0));
    t.false(isVmScript({}));
    t.false(isVmScript(lib.global._abs_));
    t.false(isVmScript((await compile('fn x {}; return x;'))()));
});
