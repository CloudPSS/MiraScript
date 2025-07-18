import test from 'ava';
import { getVmFunctionInfo, isVmFunction, VmExtern, VmFunction, VmModule, type VmAny } from 'mirascript';

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
