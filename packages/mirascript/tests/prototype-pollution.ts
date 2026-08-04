import test from 'ava';
import { compileSync, createVmContext, type VmRecord, type VmValue } from '@mirascript/mirascript';

const execute = (source: string): VmValue => compileSync(source)(createVmContext());

const expectRecord = (value: VmValue): VmRecord => value as VmRecord;

test('dangerous record keys remain own data properties', (t) => {
    const sources = [
        `(__proto__: (polluted: true))`,
        `(..(__proto__: (polluted: true)))`,
        `new_record(1, fn { ('__proto__', (polluted: true)) })`,
        `()::with('__proto__', (polluted: true))`,
        `zip((__proto__: [(polluted: true)]))[0]`,
        `map((__proto__: (polluted: true)), fn { it })`,
        `from_json('{"__proto__":{"polluted":true}}')`,
        `let input = (keep: 0, __proto__: (polluted: true)); let (:keep, ..rest) = input; rest`,
    ];

    for (const source of sources) {
        const record = expectRecord(execute(source));
        t.is(Object.getPrototypeOf(record), Object.prototype, source);
        t.true(Object.hasOwn(record, '__proto__'), source);
        t.deepEqual(record['__proto__'], { polluted: true }, source);
        t.false(Object.hasOwn(Object.prototype, 'polluted'), source);
    }
});

test('constructor.prototype paths stay inside the result record', (t) => {
    const result = expectRecord(execute(`()::with(['constructor', 'prototype', 'polluted'], true)`));
    const constructor = expectRecord(result.constructor);
    const prototype = expectRecord(constructor['prototype']!);

    t.deepEqual(prototype, { polluted: true });
    t.false(Object.hasOwn(Object.prototype, 'polluted'));
});
