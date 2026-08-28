import test from 'ava';
import {
    createVmContext,
    isVmExtern,
    type VmExtern,
    unwrapFromVmValue,
    isVmCallable,
    isVmFunction,
    VmFunction,
    getVmFunctionInfo,
} from '@mirascript/mirascript';
import { isProxy } from 'node:util/types';
import { exec } from './_exec.ts';

test('callable extern', (t) => {
    const context = createVmContext({ x: 1.2 }, { sin: Math.sin, Math: Math });
    const e = exec(context);
    t.is(e('Math::type()'), 'extern');
    t.is(e('sin::type()'), 'extern');
    t.is(e('Math.sin::type()'), 'extern');

    t.is(e('sin(x)'), Math.sin(1.2));
    t.is(e('sin(0)'), 0);
    t.is(e('Math.sin(x)'), Math.sin(1.2));
    t.is(e('Math.sin(0)'), 0);

    const eSin = e('sin') as VmExtern<Math['sin']>;
    t.true(isVmExtern(eSin));
    t.true(isVmCallable(eSin));
    t.is(eSin.value, Math.sin);
    t.is(eSin.thisArg, null);
    t.is(eSin.tag, 'function');
    t.is(unwrapFromVmValue(eSin), Math.sin);
    t.false(isProxy(unwrapFromVmValue(eSin)));
    t.is((unwrapFromVmValue(eSin) as typeof Math.sin)(1), Math.sin(1));

    const eMath = e('Math') as VmExtern<Math>;
    t.true(isVmExtern(eMath));
    t.false(isVmCallable(eMath));
    t.is(eMath.value, Math);
    t.is(eMath.thisArg, null);
    t.is(eMath.tag, 'Math');
    t.is(unwrapFromVmValue(eMath), Math);
    t.false(isProxy(unwrapFromVmValue(eMath)));

    const eMSin = e('Math.sin') as VmExtern<Math['sin']>;
    t.true(isVmExtern(eMSin));
    t.true(isVmCallable(eMSin));
    t.is(eMSin.value, Math.sin);
    t.is(eMSin.thisArg, Math);
    t.not(unwrapFromVmValue(eMSin), Math.sin);
    t.true(isProxy(unwrapFromVmValue(eMSin)));
    t.false(isProxy(unwrapFromVmValue(eMSin, false)));
    t.is((unwrapFromVmValue(eMSin) as typeof Math.sin)(1), Math.sin(1));
});

test('callable extern props', (t) => {
    const context = createVmContext({ x: 1.2 }, { sin: Math.sin, Math: Math });
    const e = exec(context);

    t.false(e('`__proto__` in sin'));
    t.false(e('`constructor` in sin'));
    t.false(e('`call` in sin'));
    t.false(e('`apply` in sin'));
    t.false(e('`bind` in sin'));
    t.false(e('`arguments` in sin'));
    t.false(e('`prototype` in sin'));
    t.false(e('`caller` in sin'));
    t.true(e('`length` in sin'));
    t.true(e('`name` in sin'));
    t.is(e('sin.name'), 'sin');

    t.false(e('`__proto__` in Math'));
    t.false(e('`constructor` in Math'));
    t.false(e('`hasOwnProperty` in Math'));
    t.false(e('`toString` in Math'));
    t.true(e('`sin` in Math'));
});

test('callback extern', (t) => {
    const cb = (a: unknown) => {
        t.is(a, cb);
    };
    const obj = {
        f(c: unknown) {
            t.is(this, obj);
            t.is(c, cb);
            return c;
        },
    };
    const context = createVmContext(null, {
        c: function (this: null, c: unknown) {
            t.is(this, null);
            t.is(c, cb);
            return c;
        },
        cb,
        proxy: new Proxy(() => 0, {
            apply(target, thisArg, args) {
                t.is(thisArg, null);
                const c = args[0];
                t.is(c, cb);
                return c;
            },
        }),
        throws: () => {
            throw new Error('Error from extern');
        },
        obj,
    });
    const e = exec(context);
    t.is(e('cb(cb)'), null);
    t.deepEqual(e('c(cb)'), e('cb'));
    t.deepEqual(e('proxy(cb)'), e('cb'));
    t.deepEqual(e('obj.f(cb)'), e('cb'));
    t.throws(() => e('throws()'), { message: /^Callable extern: Error from extern$/ });
});

test('callback native', (t) => {
    const o: Record<string, () => number> = {};
    const context = createVmContext(null, {
        o,
        c: (c: () => number) => {
            t.false(isVmFunction(c));
            const f = VmFunction(c, {
                get fullName() {
                    t.fail('fullName called');
                    return 'test';
                },
            });
            t.true(isVmFunction(f));
            t.is(getVmFunctionInfo(f)!.fullName, '');
            t.false(getVmFunctionInfo(f)!.isLib);
            t.is(c(), 1);
            t.is(f(), 1);
            return c;
        },
    });
    const e = exec(context);
    t.true(isVmFunction(e('c(fn{ 1 })')));
    t.is(e('c(fn{ 1 })::type()'), 'function');
    t.is(e('c(fn{ 1 })()'), 1);

    e('fn f{ 2 } o.f = f; o.g = f; o.h = o.g;');
    t.is(o['f'], o['g']);
    t.is(o['h'], o['g']);
    t.is(o['f'](), 2);
    t.is(VmFunction(o['f']), VmFunction(o['g']));
    t.is(VmFunction(o['h']), VmFunction(o['g']));

    e('o.i = fn { it };');
    const i = o['i'] as (value: unknown) => unknown;
    t.is(i(123), 123);
    t.is(i(i), i);
    t.is(i(o), o);
});
