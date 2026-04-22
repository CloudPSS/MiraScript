import test from 'ava';
import { lib } from '@mirascript/mirascript/subtle';

test('sparse array', (t) => {
    // eslint-disable-next-line no-sparse-arrays
    const a = [1, , 3];
    t.is(a.length, 3);
    t.is(a[0], 1);
    t.is(a[1], undefined);
    t.is(a[2], 3);

    t.deepEqual(lib.keys(a), [0, 1, 2]);
    t.deepEqual(lib.values(a), [1, undefined, 3]);
    t.deepEqual(lib.entries(a), [
        { 0: 0, 1: 1 },
        { 0: 1, 1: null },
        { 0: 2, 1: 3 },
    ]);
});
