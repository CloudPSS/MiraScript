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
    t.is(eSin.caller, null);
    t.is(eSin.describe, 'Function');
    t.is(unwrapFromVmValue(eSin), Math.sin);
    t.false(isProxy(unwrapFromVmValue(eSin)));
    t.is((unwrapFromVmValue(eSin) as typeof Math.sin)(1), Math.sin(1));

    const eMath = e('Math') as VmExtern;
    t.true(isVmExtern(eMath));
    t.is(eMath.value, Math);
    t.is(eMath.caller, null);
    t.is(eMath.describe, 'Math');
    t.is(unwrapFromVmValue(eMath), Math);
    t.false(isProxy(unwrapFromVmValue(eMath)));

    const eMSin = e('Math.sin') as VmExtern;
    t.true(isVmExtern(eMSin));
    t.is(eMSin.value, Math.sin);
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
    t.is(e('d::to_string()'), new Date(0).toString());
    t.is(e('d.toJSON()'), new Date(0).toJSON());
    t.is(e('d::to_json()'), JSON.stringify(new Date(0)));
    t.throws(() => e(`d()`), { message: /^Not a callable extern: / });

    t.is(e('construct(Date)::type()'), 'extern');
    t.is(e('construct(Date, 123).toString()'), new Date(123).toString());
    t.is(e('construct(Date, d).toString()'), new Date(0).toString());
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
        obj,
    });
    const e = exec(context);
    t.is(e('cb(cb)'), null);
    t.deepEqual(e('c(cb)'), e('cb'));
    t.deepEqual(e('proxy(cb)'), e('cb'));
    t.deepEqual(e('obj.f(cb)'), e('cb'));
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

test('extern to_string', (t) => {
    const context = createVmContext(null, {
        ok: {
            toString() {
                return 'ok';
            },
        },
        fail: {
            toString() {
                throw new Error('fail');
            },
        },
        void: {
            toString: null,
        },
        bad: {
            toString: 123,
        },
        normal: {},
        arr: [1, 2, 3],
    });
    const e = exec(context);
    t.is(e('ok::to_string()'), 'ok');
    t.is(e('fail::to_string()'), '<extern Object>');
    t.is(e('void::to_string()'), '<extern Object>');
    t.is(e('bad::to_string()'), '<extern Object>');
    t.is(e('normal::to_string()'), '<extern Object>');
    t.is(e('arr::to_string()'), '1,2,3');
});

test('extern json', (t) => {
    const context = createVmContext(null, {
        obj: { a: 1, b: 2 },
        arr: [1, 2, 3],
        func: () => 0,
        ok: {
            toJSON() {
                return { ok: true };
            },
        },
        fail: {
            toJSON() {
                throw new Error('fail');
            },
        },
        void: {
            toJSON: null,
        },
    });
    const e = exec(context);
    t.is(e('obj::to_json()'), JSON.stringify({ a: 1, b: 2 }));
    t.is(e('arr::to_json()'), JSON.stringify([1, 2, 3]));
    t.is(e('func::to_json()'), null);
    t.is(e('ok::to_json()'), JSON.stringify({ ok: true }));
    t.throws(() => e('fail::to_json()'), { message: /^Failed to convert extern to JSON: / });
    t.is(e('void::to_json()'), JSON.stringify({ toJSON: null }));
});

test('extern wrap value', (t) => {
    const context = createVmContext(null, {
        e: {
            bigint: 12_345_678_901_234_567_890n,
            symbol: Symbol('test'),
            undef: undefined,
        },
    });
    const e = exec(context);

    t.is(e('e.bigint::type()'), 'number');
    t.is(e('e.symbol::type()'), 'nil');
    t.is(e('e.undef::type()'), 'nil');

    t.is(e('e.bigint::to_json()'), JSON.stringify(Number(12_345_678_901_234_567_890n)));
    t.is(e('e.symbol::to_json()'), 'null');
    t.is(e('e.undef::to_json()'), 'null');
});

test('extern access', (t) => {
    const context = createVmContext(null, {
        obj: {
            _private: 123,
            visible: 456,
            method() {
                return 789;
            },
            __proto__: {
                // eslint-disable-next-line @typescript-eslint/unbound-method
                toString: Object.prototype.toString,
            },
        },
        func: function () {
            return 0;
        },
        arr: Object.setPrototypeOf([1, 2, 3], {
            map: 12,
            sort: Array.prototype.sort,
        }),
    });
    const e = exec(context);

    t.false(e('`_private` in obj'));
    t.is(e('obj._private'), null);
    t.is(e('obj.visible'), 456);
    t.is(e('obj.method::type()'), 'extern');
    t.is(e('obj.method()'), 789);

    t.false(e('`__proto__` in obj'));
    t.false(e('`toString` in obj'));

    t.false(e('`prototype` in func'));
    t.false(e('`arguments` in func'));
    t.false(e('`caller` in func'));

    t.true(e('`map` in arr'));
    t.false(e('`sort` in arr'));
    t.false(e('`filter` in arr'));
    t.true(e('`length` in arr'));
    t.is(e('arr.length'), 3);
    t.is(e('arr[0]'), 1);
    t.is(e('arr[1]'), 2);
    t.is(e('arr[2]'), 3);

    t.is(e('arr.map::type()'), 'number');
    t.is(e('arr.sort::type()'), 'nil');
});

test('extern iterable', (t) => {
    const context = createVmContext(null, {
        arr: [10, 20, 30],
        map: new Map([
            ['a', 1],
            ['b', 2],
            ['c', 3],
        ]),
        set: new Set([100, 200, 300]),
        noniter: {
            a: 1,
            b: 2,
        },
    });
    const e = exec(context);

    t.deepEqual(e('[..arr]'), [10, 20, 30]);
    t.deepEqual(e('[..map]'), [null, null, null]);
    t.deepEqual(e('[..map.keys()]'), ['a', 'b', 'c']);
    t.deepEqual(e('[..map.values()]'), [1, 2, 3]);
    t.deepEqual(e('[..set]'), [100, 200, 300]);
    t.throws(() => e('[..noniter]'), { message: 'Expected array, iterable extern or nil, got extern' });
});

test('extern spread', (t) => {
    const context = createVmContext(null, {
        obj: { a: 1, b: 2, n: undefined },
        arr: [3, 4, 5],
    });
    const e = exec(context);

    t.deepEqual(e('(..obj, c: 3)'), { a: 1, b: 2, c: 3, n: null });
    t.deepEqual(e('(..arr)'), { 0: 3, 1: 4, 2: 5 });
});
