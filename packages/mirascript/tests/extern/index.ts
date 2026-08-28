import test from 'ava';
import { createVmContext, VmExtern, unwrapFromVmValue } from '@mirascript/mirascript';
import { exec } from './_exec.ts';

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
