import test from 'ava';
import { operations } from '@mirascript/mirascript/subtle';
const { $Same, $ArrayRange, $ArrayRangeExclusive } = operations;

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
    t.notThrows(() => $ArrayRange(0, 134_217_727));
    t.notThrows(() => $ArrayRangeExclusive(0, 134_217_728));
    t.throws(() => $ArrayRange(0, 134_217_728), {
        instanceOf: RangeError,
        message: /^Array length exceeds maximum limit/,
    });
    t.throws(() => $ArrayRangeExclusive(0, 134_217_729), {
        instanceOf: RangeError,
        message: /^Array length exceeds maximum limit/,
    });
});
