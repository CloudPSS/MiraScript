import test from 'ava';
import { parse } from '@mirascript/typed';

test('union type', (t) => {
    t.deepEqual(parse('string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.deepEqual(parse('true | false'), {
        kind: 'union',
        types: [
            { kind: 'literal', value: true },
            { kind: 'literal', value: false },
        ],
    });
});

test('union type with leading pipe', (t) => {
    t.deepEqual(parse('| string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
});

test('intersection type', (t) => {
    t.deepEqual(parse('A & B'), {
        kind: 'intersection',
        types: ['A', 'B'],
    });
    t.deepEqual(parse('true & false'), {
        kind: 'intersection',
        types: [
            { kind: 'literal', value: true },
            { kind: 'literal', value: false },
        ],
    });
});

test('intersection type with leading ampersand', (t) => {
    t.deepEqual(parse('& A & B'), {
        kind: 'intersection',
        types: ['A', 'B'],
    });
});
