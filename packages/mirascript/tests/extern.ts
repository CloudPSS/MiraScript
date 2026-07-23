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
    isVmCallable,
    type VmArray,
    type VmRecord,
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
    t.is(new VmExtern({}, null).tag, 'Object');
    t.is(new VmExtern(Object.create(null), null).tag, 'Object: null prototype');
    t.is(new VmExtern([], null).tag, 'Array(0)');
    t.is(new VmExtern(() => 0, null).tag, 'function');
    // eslint-disable-next-line @typescript-eslint/require-await
    t.is(new VmExtern(async () => 0, null).tag, 'async function');
    t.is(
        new VmExtern(function* () {
            yield 0;
        }, null).tag,
        'function*',
    );
    t.is(
        // eslint-disable-next-line @typescript-eslint/require-await
        new VmExtern(async function* () {
            yield 0;
        }, null).tag,
        'async function*',
    );
    const a = class A {
        x = 1;
    };
    t.is(new VmExtern(new a(), null).tag, 'Object');
    t.is(new VmExtern(a, null).tag, 'class');
    Object.defineProperty(a, 'name', { value: 'ALongName' });
    t.is(new VmExtern(new a(), null).tag, 'ALongName');
    t.is(new VmExtern(a, null).tag, 'class ALongName');
    const ab = a.bind(null);
    t.is(new VmExtern(new ab(), null).tag, 'ALongName');
    t.is(new VmExtern(ab, null).tag, 'function');
    Object.defineProperty(a, 'name', { value: '' });
    t.is(new VmExtern(new a(), null).tag, 'Object');
    t.is(new VmExtern(a, null).tag, 'class');
    Object.defineProperty(a, 'displayName', { value: 'ADisplayName' });
    t.is(new VmExtern(new a(), null).tag, 'ADisplayName');
    t.is(new VmExtern(a, null).tag, 'class ADisplayName');
    // eslint-disable-next-line unicorn/consistent-function-scoping
    const f = function () {
        return 1;
    };
    t.is(new VmExtern(f, null).tag, 'class');
    f.prototype = undefined;
    t.is(new VmExtern(f, null).tag, 'function');
    f.prototype = null;
    t.is(new VmExtern(f, null).tag, 'class');
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
    t.is(e('d::type()'), 'number');
    t.is(e('construct::type()'), 'extern');

    t.false(e('`prototype` in Date'));

    t.is(e('construct(Date)::type()'), 'number');
    t.is(e('construct(Date, 123)'), 123);
    t.is(e('construct(Date, d)'), 0);
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
        arr: [
            1,
            2,
            3,
            /test/i,
            undefined,
            null,
            [1, 2, 3],
            {
                toString() {
                    return `Custom String`;
                },
            },
            // eslint-disable-next-line no-sparse-arrays
            ,
        ],
        fail_arr: [
            {
                toString() {
                    throw new Error('fail');
                },
            },
        ],
    });
    const e = exec(context);
    t.is(e('ok::to_string()'), 'ok');
    t.throws(() => e('fail::to_string()'), { message: 'Failed to convert value to string: <extern>' });
    t.is(e('fail::to_string(0)'), 0);
    t.is(e('void::to_string()'), '<extern Object>');
    t.is(e('bad::to_string()'), '<extern Object>');
    t.is(e('normal::to_string()'), '<extern Object>');
    t.is(e('arr::to_string()'), '1, 2, 3, /test/i, , nil, [1, 2, 3], Custom String, ');
    t.throws(() => e('fail_arr::to_string()'), { message: 'Failed to convert value to string: <extern>' });
    t.is(e('arr.3::to_string()'), '/test/i');
});

test('extern json', (t) => {
    const context = createVmContext(null, {
        obj: { a: 1, b: 2 },
        arr: [1, 2, 3],
        func: () => 0,
        func_json: Object.assign(() => 0, {
            toJSON: () => 'func_json',
        }),
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
    t.is(e('func_json::to_json()'), '"func_json"');
    t.is(e('ok::to_json()'), JSON.stringify({ ok: true }));
    t.throws(() => e('fail::to_json()'), { message: /^fail$/ });
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

    // Write access
    t.is(e('obj._p = (); obj._p'), null);
    t.is(e('obj.p = 12; obj.p'), 12);
    t.is(e('obj.toString = "xx"; obj.toString'), 'xx');
    t.is(e('obj.prototype = "xx"; obj.prototype'), 'xx');
    t.is(e('func.prototype = "xx"; func.prototype'), null);
    t.is(e('func.xx = 123; func.xx'), 123);
    t.is(e('arr.map = "xx"; arr.map'), 'xx');
    t.is(e('arr.12 = 12; arr.12'), 12);
    t.is(e('arr.length'), 13);
    t.is(e('arr.11'), null);
});

test('extern iterable', (t) => {
    const context = createVmContext(null, {
        arr: [10, 20, 30],
        map: new Map([
            ['a', new Date(1)],
            ['b', new Date(2)],
            ['c', new Date(3)],
        ]),
        set: new Set([100, 200, 300]),
        noniter: {
            a: 1,
            b: 2,
        },
        tarr: new Uint16Array([1, 2, 3]),
    });
    const e = exec(context);

    t.deepEqual(e('[..arr]'), [10, 20, 30]);
    t.deepEqual(e('arr[-1]'), 30);
    t.deepEqual(e('arr.3'), null);
    t.deepEqual(e('arr[3]'), null);
    t.deepEqual(e('arr[2.99]'), 30);
    t.deepEqual(e('arr["2.99"]'), null);
    t.deepEqual(e('arr[-3]'), 10);
    t.deepEqual(e('arr["-3"]'), null);
    t.deepEqual(e('arr[-4]'), null);
    t.deepEqual(e('[..tarr]'), [1, 2, 3]);
    t.deepEqual(e('tarr[-1]'), 3);
    t.deepEqual(e('[..map]'), [null, null, null]);
    t.deepEqual(e('map.get("a")'), 1);
    t.deepEqual(e('[..map.keys()]'), ['a', 'b', 'c']);
    t.deepEqual(e('[..map.values()]'), [1, 2, 3]);
    t.deepEqual(e('[..set]'), [100, 200, 300]);
    t.deepEqual(e('arr::len()'), 3);
    t.deepEqual(e('tarr::len()'), 3);
    t.throws(() => e('set::len()'), { message: "Argument 'arr' is not array-like extern: <extern Set>" });
    t.throws(() => e('arr[1..2]'), { message: 'Expected array, got <extern Array(3)> [10, 20, 30]' });
    t.throws(() => e('[..noniter]'), { message: 'Expected array, iterable extern or nil, got <extern Object>' });
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

test('extern keys', (t) => {
    const v = { __proto__: Math, _private: 123, public: 456 };
    const e = new VmExtern(v, null);
    const keys = e.keys();
    t.deepEqual(keys.sort(), ['public']);
    const allKeys = e.keys(true);
    t.deepEqual(allKeys.sort(), ['public', ...Object.getOwnPropertyNames(Math)].sort());
    for (const key of allKeys) {
        t.true(e.has(key));
        t.is(unwrapFromVmValue(e.get(key), false), v[key as keyof typeof v]);
    }
});

test('custom extern', (t) => {
    class MyExtern extends VmExtern {
        override assumeVmValue(value: object, key: undefined): value is VmRecord | VmArray {
            return true;
        }
    }
    const context = createVmContext(
        {
            my: new MyExtern({ a: [], b: {}, f: VmFunction(() => 0) }),
        },
        {
            vm: { a: [], b: {}, f: VmFunction(() => 0) },
        },
    );
    const e = exec(context);

    t.is(e('my::type()'), 'extern');
    t.is(e('vm::type()'), 'extern');

    t.is(e('my.a::type()'), 'array');
    t.is(e('my.b::type()'), 'record');
    t.is(e('my.f::type()'), 'function');

    t.is(e('vm.a::type()'), 'extern');
    t.is(e('vm.b::type()'), 'extern');
    t.is(e('vm.f::type()'), 'function');
});
