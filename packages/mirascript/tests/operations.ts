import test from 'ava';
import { VmFunction, VmModule, type VmValue, VmExtern } from '@mirascript/mirascript';
import { operations } from '@mirascript/mirascript/subtle';
const { $Same, $ArrayRange, $ArrayRangeExclusive, $Omit, $Pick } = operations;

test('compare objects with non-enumerable properties', (t) => {
    const obj1 = { a: 1 };
    const obj2 = Object.defineProperty({ b: 2 }, 'a', { value: 1, enumerable: false });
    t.is($Same(obj1, obj2), false);
    t.is($Same(obj2, obj1), false);
});

test('compare objects with undefined prototypes', (t) => {
    const obj1 = { x: undefined };
    const obj2 = Object.create(null);
    t.is($Same(obj1, obj2), false);
    t.is($Same(obj2, obj1), false);
});

test('compare objects with undefined and null prototypes', (t) => {
    const obj1 = { x: undefined };
    const obj2 = { x: null };
    t.is($Same(obj1, obj2), true);
    t.is($Same(obj2, obj1), true);
});

test('array range should throw on excessive length', (t) => {
    t.notThrows(() => $ArrayRange(0, 16_777_215));
    t.notThrows(() => $ArrayRangeExclusive(0, 16_777_216));
    t.throws(() => $ArrayRange(0, 16_777_216), {
        instanceOf: RangeError,
        message: /^Array length exceeds maximum limit/,
    });
    t.throws(() => $ArrayRangeExclusive(0, 16_777_217), {
        instanceOf: RangeError,
        message: /^Array length exceeds maximum limit/,
    });
});

test('omit and pick should return empty object for non-record values', (t) => {
    const nonRecordValues: VmValue[] = [
        null,
        42,
        'string',
        true,
        [],
        VmFunction(() => 0),
        new VmModule('x', {}),
        new VmExtern(new Date()),
    ];
    for (const value of nonRecordValues) {
        t.deepEqual($Omit(value, []), {});
        t.deepEqual($Pick(value, []), {});
    }
});

test('omit and pick should work with non-enumerable properties', (t) => {
    const obj = Object.defineProperty({ a: 1, b: 2 }, 'c', { value: 3, enumerable: false });
    const omitted = $Omit(obj, ['a']);
    t.deepEqual(omitted, { b: 2 });
    const picked = $Pick(obj, ['b', 'c']);
    t.deepEqual(picked, { b: 2 });
});

test('omit and pick should work with undefined prototypes', (t) => {
    const obj: Record<string, number> = Object.create(null);
    obj['a'] = 1;
    obj['b'] = 2;
    const omitted = $Omit(obj, ['a']);
    t.deepEqual(omitted, { b: 2 });
    t.is(Object.getPrototypeOf(omitted), Object.prototype);
    const picked = $Pick(obj, ['b']);
    t.deepEqual(picked, { b: 2 });
    t.is(Object.getPrototypeOf(picked), Object.prototype);
});

test('omit and pick should work with __proto__ property', (t) => {
    const obj = { a: 1, b: 2, ['__proto__']: { c: 3 } };
    const omitted = $Omit(obj, ['a']);
    t.deepEqual(omitted, { b: 2, ['__proto__']: { c: 3 } });
    t.is(Object.getPrototypeOf(omitted), Object.prototype);
    const picked = $Pick(obj, ['b', '__proto__']);
    t.deepEqual(picked, { b: 2, ['__proto__']: { c: 3 } });
    t.is(Object.getPrototypeOf(picked), Object.prototype);
});

test('omit and pick should work with numeric property', (t) => {
    const obj = { 1: 'one', 2: 'two', 3: 'three' };
    const omitted = $Omit(obj, [2]);
    t.deepEqual(omitted, { 1: 'one', 3: 'three' });
    const picked = $Pick(obj, [1, 3]);
    t.deepEqual(picked, { 1: 'one', 3: 'three' });
});
