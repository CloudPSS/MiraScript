import test from 'ava';
import { print } from '../dist/cli/print.js';
import { execute } from '../dist/cli/execute.js';
import { createVmContext } from '../dist/index.js';

test('cli print', (t) => {
    t.snapshot(print(null));
    t.snapshot(print(undefined));
    t.snapshot(print(true));
    t.snapshot(print(false));
    t.snapshot(print(0));
    t.snapshot(print(Number.NEGATIVE_INFINITY));
    t.snapshot(print(Number.POSITIVE_INFINITY));
    t.snapshot(print(Number.NaN));
    t.snapshot(print('Hello, world!'));
    const context = createVmContext();
    t.snapshot(print(context.get('matrix')));
    t.snapshot(print(context.get('sin')));
});

test('cli execute', async (t) => {
    t.is(await execute('true', false, {}), undefined);
    t.is(await execute('true', true, {}), undefined);
    t.is(await execute('debug_print(1, "x", sin)', false, {}), undefined);
});
