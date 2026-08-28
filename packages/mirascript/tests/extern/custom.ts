import test from 'ava';
import { createVmContext, VmExtern, VmFunction, type VmArray, type VmRecord } from '@mirascript/mirascript';
import { exec } from './_exec.ts';

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
