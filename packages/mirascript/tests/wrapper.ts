import test from 'ava';
import {
    compile,
    createVmContext,
    getVmFunctionInfo,
    isVmFunction,
    VmExtern,
    VmFunction,
    VmModule,
    type VmAny,
} from '@mirascript/mirascript';

test('toJSON', (t) => {
    t.deepEqual(JSON.stringify(new VmExtern({})), undefined);
    t.deepEqual(JSON.stringify(new VmModule('test', {})), undefined);
});

test('VmFunction', (t) => {
    const fn = (a: VmAny, b: VmAny) => Number(a) + Number(b);
    t.is(getVmFunctionInfo(VmFunction(fn, { injectCp: true }))?.original, fn);

    const vmFn = VmFunction(fn);
    t.is(vmFn, fn as VmFunction<(a: VmAny, b: VmAny) => number>);
    t.deepEqual(vmFn(1, 2), 3);
    t.true(isVmFunction(vmFn));
    t.is(vmFn, VmFunction(vmFn));
    t.is(getVmFunctionInfo(vmFn)?.fullName, 'fn');
    t.is(getVmFunctionInfo(vmFn)?.original, undefined);

    t.throws(() => VmFunction(123 as never), { instanceOf: TypeError });
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
