import test from 'ava';
import program from '../dist/cli/index.js';
import { print } from '../dist/cli/print.js';
import { execute } from '../dist/cli/execute.js';
import { compile, createVmContext, VmModule } from '../dist/index.js';

test('cli program', (t) => {
    t.is(program.name(), 'mirascript');
    t.regex(program.version()!, /^\d+\.\d+\.\d+(-.+)?$/);
    t.is(typeof program.description(), 'string');
});

test('cli print', async (t) => {
    t.snapshot(print(null));
    t.snapshot(print(undefined));
    t.snapshot(print(true));
    t.snapshot(print(false));
    t.snapshot(print(0));
    t.snapshot(print(-0));
    t.snapshot(print(Number.NEGATIVE_INFINITY));
    t.snapshot(print(Number.POSITIVE_INFINITY));
    t.snapshot(print(Number.NaN));
    t.snapshot(print([+0, -0, Number.NaN]));
    t.snapshot(print('Hello, world!'));
    t.snapshot(print('Hello, world!\nNew line.\tTabbed.\\Backslash. "Double quotes". \'Single quotes\'.'));
    const context = createVmContext();
    t.snapshot(print(context.get('matrix')));
    t.snapshot(print(context.get('sin')));
    t.snapshot(print((await compile('fn a{}a'))(context)));
    t.snapshot(print(new VmModule('', {})));
});

test('cli execute', async (t) => {
    t.is(await execute('true', false, {}, ''), undefined);
    t.is(await execute('true', true, {}, ''), undefined);
    t.is(await execute('debug_print(1, "x", sin)', false, {}, ''), undefined);
});
