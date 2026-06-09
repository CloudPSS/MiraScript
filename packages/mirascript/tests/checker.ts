import test from 'ava';
import {
    compile,
    isVmScript,
    isVmFunction,
    isVmAny,
    isVmValue,
    isVmImmutable,
    isVmConst,
    isVmContext,
    createVmContext,
    VmModule,
    VmExtern,
    isVmCallable,
    type VmRecord,
    isVmArrayLikeRecord,
    isVmArrayLikeRecordByEntires,
    isVmArrayLikeRecordByKeys,
    getVmType,
} from '@mirascript/mirascript';
import { DefaultVmContext, lib } from '@mirascript/mirascript/subtle';

test('isVmScript', async (t) => {
    t.true(isVmScript(await compile('nil')));
    t.true(isVmScript(await compile('1 + 1')));
    t.false(isVmScript(() => 0));
    t.false(isVmScript({}));
    t.false(isVmScript((await compile('abs'))()));
    t.false(isVmScript(lib.abs));
    t.false(isVmScript((await compile('fn x {}; return x;'))()));
});

test('isVmFunction', async (t) => {
    t.false(isVmFunction(await compile('1 + 1')));
    t.false(isVmFunction(() => 0));
    t.false(isVmFunction({}));
    t.true(isVmFunction((await compile('abs'))()));
    t.true(isVmFunction(lib.abs));
    t.true(isVmFunction(lib.from_json));
});

test('isVmCallable', async (t) => {
    t.false(isVmCallable(await compile('1 + 1')));
    t.false(isVmCallable({}));
    t.true(isVmCallable((await compile('abs'))()));
    t.true(isVmCallable(new VmExtern(() => 0)));
    t.false(isVmCallable(new VmExtern({})));
    t.true(isVmCallable(lib.abs));
    t.true(isVmCallable(lib.from_json));
});

test('isVmContext', (t) => {
    t.false(isVmContext(null));
    t.false(isVmContext(() => null));
    t.false(isVmContext({}));
    t.false(isVmContext({ __proto__: null }));

    t.true(isVmContext(DefaultVmContext));
    t.true(isVmContext(createVmContext()));
    t.true(isVmContext(createVmContext(() => 0)));
    t.true(isVmContext(createVmContext({ a: 1 })));
});

test('isVmAny', (t) => {
    t.true(isVmAny(1, false));
    t.true(isVmAny('test', false));
    t.true(isVmAny(true, false));
    t.true(isVmAny(null, false));
    t.true(isVmAny(undefined, false));
    t.true(isVmAny({}, false));
    t.true(isVmAny([], false));
    t.false(isVmAny(() => 0, false));
    t.false(isVmAny(Symbol('test'), false));
    t.false(isVmAny(new Map(), false));
    t.false(isVmAny(/r/, false));
    t.false(isVmAny(0n, false));
    t.true(isVmAny([new Map()], false));
    t.true(isVmAny([/r/], false));
    t.true(isVmAny([0n], false));
});

test('isVmArrayLikeRecord', (t) => {
    const r1 = { '0': 1, '1': 2, '2': 3 };
    const r2 = { '0': 1, '2': 3 };
    const r3 = { '1': 2, '2': 3 };
    const r4 = { a: 1, b: 2 };
    const r5 = {};
    const r6 = { '1': 1, '0': 2 };
    const r7: Record<string, number> = Object.create(null);
    r7['1'] = 2;
    r7['0'] = 1;
    const r8 = { '0.0': 1 };
    const r9 = { '0': 1, '1.2': 2, '2': 3 };
    const r10 = { '0': 1, '1': 2, '2': 3, length: 3 };

    const test = (r: VmRecord, ex: boolean) => {
        t.is(isVmArrayLikeRecord(r), ex, JSON.stringify(r));
        t.is(isVmArrayLikeRecordByKeys(Object.keys(r)), ex);
        t.is(isVmArrayLikeRecordByEntires(Object.entries(r)), ex);
    };

    test(r1, true);
    test(r2, false);
    test(r3, false);
    test(r4, false);
    test(r5, true);
    test(r6, true);
    test(r7, true);
    test(r8, false);
    test(r9, false);
    test(r10, false);

    // eslint-disable-next-line unicorn/no-new-array
    const vl: readonly never[] = new Array(2 ** 31);
    t.false(isVmArrayLikeRecordByKeys(vl));
    t.false(isVmArrayLikeRecordByEntires(vl));
});

test('isVmValue', (t) => {
    t.true(isVmValue(1));
    t.true(isVmValue('test'));
    t.true(isVmValue(true));
    t.true(isVmValue(null));
    t.false(isVmValue(undefined));
    t.true(isVmValue({}));
    t.true(isVmValue([]));
    t.true(isVmValue(new VmModule('test', {})));
    t.true(isVmValue(new VmExtern({})));

    t.false(isVmValue(() => 0, false));
    t.false(isVmValue(Symbol('test'), false));
    t.false(isVmValue(new Map(), false));
    t.false(isVmValue(/r/, false));
    t.false(isVmValue(0n, false));
});

test('isVmImmutable', async (t) => {
    t.true(isVmImmutable(1));
    t.true(isVmImmutable('test'));
    t.true(isVmImmutable(true));
    t.true(isVmImmutable(null));
    t.false(isVmImmutable(undefined));
    t.true(isVmImmutable({}));
    t.true(isVmImmutable([]));
    t.false(isVmImmutable(() => 0, false));
    t.true(isVmImmutable((await compile('abs'))()));
    t.false(isVmImmutable(Symbol('test'), false));
    t.false(isVmImmutable(new Map(), false));
    t.false(isVmImmutable(/r/, false));
    t.false(isVmImmutable(0n, false));
    t.true(isVmImmutable(new VmModule('test', {})));
    t.false(isVmImmutable(new VmExtern({})));
});

test('isVmConst', async (t) => {
    t.true(isVmConst(1));
    t.true(isVmConst('test'));
    t.true(isVmConst(true));
    t.true(isVmConst(null));
    t.false(isVmConst(undefined));
    t.true(isVmConst({}));
    t.true(isVmConst([]));
    t.false(isVmConst((await compile('abs'))()));
    t.false(isVmConst(new VmModule('test', {})));
    t.false(isVmConst(new VmExtern({})));
});

test('isVmConst with deep check', async (t) => {
    t.true(isVmConst(1, true));
    t.true(isVmConst('test', true));
    t.true(isVmConst(true, true));
    t.true(isVmConst(null, true));
    t.false(isVmConst(undefined, true));
    t.true(isVmConst({ a: 1 }, true));
    t.true(isVmConst([1, 2, 3], true));
    t.false(isVmConst((await compile('abs'))(), true));
    t.false(isVmConst(new VmModule('test', {}), true));
    t.false(isVmConst(new VmExtern({}), true));
    t.false(isVmConst([new Map()], true));
    t.false(isVmConst([/r/], true));
    t.false(isVmConst([0n], true));
    t.true(isVmConst(['x', 'y', [1, 2, 3]], true));
    t.false(isVmConst([() => 0], true));
    t.true(isVmConst([undefined], true));
    // eslint-disable-next-line no-sparse-arrays
    t.true(isVmConst([, , ,], true));
    t.true(isVmConst({ a: undefined }, true));
    t.true(isVmConst({ a: null }, true));
    t.false(isVmConst({ a: new VmModule('test', {}) }, true));
    t.true(isVmConst({ __proto__: null }, true));
    // eslint-disable-next-line unicorn/new-for-builtins
    t.false(isVmConst(new Number(1), true));

    class SubArr extends Array<number> {}
    t.false(isVmConst([new SubArr(1, 2, 3)], true));
    Object.setPrototypeOf(SubArr.prototype, Object.prototype);
    t.false(isVmConst([new SubArr(1, 2, 3)], true));
    Object.setPrototypeOf(SubArr.prototype, null);
    t.false(isVmConst([new SubArr(1, 2, 3)], true));

    const x: unknown[] = [];
    Object.setPrototypeOf(x, null);
    t.false(isVmConst(x, true));

    let deep: unknown[] = [];
    for (let i = 0; i < 1000; i++) {
        deep = [deep];
    }
    t.false(isVmConst(deep, true));
});

test('getVmType', async (t) => {
    t.is(getVmType(1), 'number');
    t.is(getVmType('test'), 'string');
    t.is(getVmType(true), 'boolean');
    t.is(getVmType(null), 'nil');
    t.is(getVmType(undefined), 'uninitialized');
    t.is(getVmType({}), 'record');
    t.is(getVmType([]), 'array');
    t.is(getVmType(new VmModule('test', {})), 'module');
    t.is(getVmType(new VmExtern({})), 'extern');
    t.is(getVmType((await compile('abs'))()), 'function');
});
