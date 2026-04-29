import test from 'ava';
import {
    compile,
    createVmContext,
    getVmFunctionInfo,
    isVmFunction,
    VmFunction,
    VmModule,
    type VmAny,
} from '@mirascript/mirascript';

test('VmFunction', (t) => {
    const fn = (a: VmAny, b: VmAny) => Number(a) + Number(b);

    const vmFn = VmFunction(fn);
    t.is(vmFn, fn as VmFunction<typeof fn>);
    t.deepEqual(vmFn(1, 2), 3);
    t.true(isVmFunction(vmFn));
    t.is(vmFn, VmFunction(vmFn));
    t.is(vmFn.name, 'fn');
    t.is(getVmFunctionInfo(vmFn)?.fullName, 'fn');

    t.throws(() => VmFunction(123 as never), { instanceOf: TypeError });

    const namedFn = VmFunction((a, b) => Number(a) + Number(b), { name: 'add', fullName: 'math.add' });
    t.is(getVmFunctionInfo(namedFn)?.fullName, 'math.add');
    t.is(namedFn.name, 'add');

    const recreateFn = VmFunction((a: VmAny, b: VmAny) => Number(a) * Number(b), namedFn);
    t.not(recreateFn, fn);
    t.deepEqual(recreateFn(1, 2), 2);
    t.true(isVmFunction(recreateFn));
    t.is(recreateFn, VmFunction(recreateFn));
    t.is(recreateFn.name, 'add');
    t.is(getVmFunctionInfo(recreateFn)?.fullName, 'math.add');
});

test('VmModule', async (t) => {
    const module = new VmModule('test', { a: 1, b: 2, c: undefined as never });
    t.is(module.name, 'test');
    const context = createVmContext({ test: module });
    t.is(module.describe('a'), undefined);
    t.is(context.get('test'), module);
    t.is((await compile('test.a'))(context), 1);
    t.is((await compile('test.c'))(context), null);
    t.is((await compile('test.ne'))(context), null);
    t.true((await compile('`a` in test'))(context));
    t.true((await compile('`c` in test'))(context));
    t.false((await compile('`ne` in test'))(context));
});
