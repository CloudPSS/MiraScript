import test from 'ava';
import {
    compileSync,
    createVmContext,
    isVmExtern,
    VmExtern,
    type VmContext,
    type VmValue,
    isVmFunction,
    VmFunction,
    getVmFunctionInfo,
    unwrapFromVmValue,
} from '@mirascript/mirascript';
import { isProxy } from 'node:util/types';

const exec = (context: VmContext): ((source: string) => VmValue) => {
    return (source: string) => {
        const script = compileSync(source);
        return script(context);
    };
};

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

    const eSin = e('sin') as VmExtern;
    t.true(isVmExtern(eSin));
    t.is(eSin.value, Math.sin);
    t.true(eSin.callable);
    t.is(eSin.caller, null);
    t.is(eSin.describe, 'Function');
    t.is(unwrapFromVmValue(eSin), Math.sin);
    t.false(isProxy(unwrapFromVmValue(eSin)));
    t.is((unwrapFromVmValue(eSin) as typeof Math.sin)(1), Math.sin(1));

    const eMath = e('Math') as VmExtern;
    t.true(isVmExtern(eMath));
    t.is(eMath.value, Math);
    t.false(eMath.callable);
    t.is(eMath.caller, null);
    t.is(eMath.describe, 'Math');
    t.is(unwrapFromVmValue(eMath), Math);
    t.false(isProxy(unwrapFromVmValue(eMath)));

    const eMSin = e('Math.sin') as VmExtern;
    t.true(isVmExtern(eMSin));
    t.is(eMSin.value, Math.sin);
    t.true(eMSin.callable);
    t.is(eMSin.caller, eMath);
    t.true(eMSin.caller!.same(eMath));
    t.not(unwrapFromVmValue(eMSin), Math.sin);
    t.true(isProxy(unwrapFromVmValue(eMSin)));
    t.is((unwrapFromVmValue(eMSin) as typeof Math.sin)(1), Math.sin(1));

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
});

test('describe extern', (t) => {
    t.is(new VmExtern({}, null).describe, 'Object');
    t.is(new VmExtern(Object.create(null), null).describe, 'Object: null prototype');
    t.is(new VmExtern([], null).describe, 'Array');
    t.is(new VmExtern(() => 0, null).describe, 'Function');
    // eslint-disable-next-line @typescript-eslint/require-await
    t.is(new VmExtern(async () => 0, null).describe, 'AsyncFunction');
    t.is(
        new VmExtern(function* () {
            yield 0;
        }, null).describe,
        'GeneratorFunction',
    );
    t.is(
        // eslint-disable-next-line @typescript-eslint/require-await
        new VmExtern(async function* () {
            yield 0;
        }, null).describe,
        'AsyncGeneratorFunction',
    );
    const a = class A {
        x = 1;
    };
    t.is(new VmExtern(new a(), null).describe, 'A');
    t.is(new VmExtern(a, null).describe, 'Class A');
    Object.defineProperty(a, 'name', { value: '' });
    t.is(new VmExtern(new a(), null).describe, 'Object');
    t.is(new VmExtern(a, null).describe, 'Class');
    // eslint-disable-next-line unicorn/consistent-function-scoping
    const f = function () {
        return 1;
    };
    t.is(new VmExtern(f, null).describe, 'Class f');
    f.prototype = undefined;
    t.is(new VmExtern(f, null).describe, 'Function');
    f.prototype = null;
    t.is(new VmExtern(f, null).describe, 'Class f');
});

test('Date extern', (t) => {
    const context = createVmContext(null, {
        Date: Date,
        d: new Date(0),
        construct: (c: unknown, ...args: unknown[]) => {
            return new (c as new (...args: unknown[]) => unknown)(...args);
        },
    });
    const e = exec(context);
    t.is(e('Date::type()'), 'extern');
    t.is(e('d::type()'), 'extern');

    t.false(e('`prototype` in Date'));
    t.false(e('`constructor` in d'));

    t.true(e('`toString` in d'));
    t.is(e('d.toString()'), new Date(0).toString());
    t.throws(() => e(`d()`), { message: /Expected callable/ });

    t.is(e('construct(Date)::type()'), 'extern');
    t.is(e('construct(Date, 123).toString()'), new Date(123).toString());
    t.is(e('construct(Date, d).toString()'), new Date(0).toString());
});

test('callback extern', (t) => {
    const cb = (a: unknown) => {
        t.is(a, cb);
    };
    const context = createVmContext(null, {
        c: (c: unknown) => {
            t.is(c, cb);
            return c;
        },
        cb,
    });
    const e = exec(context);
    t.is(e('cb(cb)'), null);
    t.deepEqual(e('c(cb)'), e('cb'));
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
    t.is(o['f']!(), 2);
    t.is(VmFunction(o['f']!), VmFunction(o['g']!));
    t.is(VmFunction(o['h']!), VmFunction(o['g']!));

    e('o.i = fn { it };');
    const i = o['i']! as (value: unknown) => unknown;
    t.is(i(123), 123);
    t.is(i(i), i);
    t.is(i(o), o);
});
