import test from 'ava';
import { parse, simplify } from '@mirascript/typed';

test('simplify preserves reflection types', (t) => {
    t.deepEqual(simplify(parse('type(MyVar)')), {
        kind: 'reflection',
        name: 'MyVar',
    });
    t.deepEqual(simplify(parse('type(MyVar) | string')), {
        kind: 'union',
        types: [{ kind: 'reflection', name: 'MyVar' }, 'string'],
    });
});

test('simplify top types in unions', (t) => {
    t.is(simplify(parse('unknown | string')), 'unknown');
    t.is(simplify(parse('any | string')), 'any');
    t.is(simplify(parse('string | never')), 'string');
    t.deepEqual(simplify(parse('(string | never) | number')), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.is(simplify(parse('never | never')), 'never');
    t.is(simplify(parse('never | any')), 'any');
    t.is(simplify(parse('never | unknown')), 'unknown');
    t.is(simplify(parse('unknown | unknown')), 'unknown');
    t.is(simplify(parse('unknown | any')), 'any');
    t.is(simplify(parse('any | any')), 'any');
});

test('simplify top types in intersections', (t) => {
    t.is(simplify(parse('never & string')), 'never');
    t.is(simplify(parse('string & unknown')), 'string');
    t.is(simplify(parse('string & any')), 'string');
    t.deepEqual(simplify(parse('(string & any) & number')), {
        kind: 'intersection',
        types: ['string', 'number'],
    });
    t.is(simplify(parse('never & never')), 'never');
    t.is(simplify(parse('never & any')), 'never');
    t.is(simplify(parse('never & unknown')), 'never');
    t.is(simplify(parse('unknown & unknown')), 'unknown');
    t.is(simplify(parse('unknown & any')), 'any');
    t.is(simplify(parse('any & any')), 'any');
});

test('simplify top types option as array', (t) => {
    // Only absorb unknown in unions
    t.is(simplify(parse('unknown | string'), { simplifyTopTypesInUnions: ['unknown'] }), 'unknown');
    t.deepEqual(simplify(parse('any | string'), { simplifyTopTypesInUnions: ['unknown'] }), {
        kind: 'union',
        types: ['any', 'string'],
    });
    // never is not in the list, so it stays — verify roundtrip
    t.deepEqual(simplify(parse('string | never'), { simplifyTopTypesInUnions: ['unknown'] }), {
        kind: 'union',
        types: ['string', 'never'],
    });

    // Only eliminate never in intersections
    t.is(simplify(parse('never & string'), { simplifyTopTypesInIntersections: ['never'] }), 'never');
    t.deepEqual(simplify(parse('string & unknown'), { simplifyTopTypesInIntersections: ['never'] }), {
        kind: 'intersection',
        types: ['string', 'unknown'],
    });
});

test('simplify top types option disabled', (t) => {
    // With false, top types are left untouched
    t.deepEqual(simplify(parse('unknown | string'), { simplifyTopTypesInUnions: false }), {
        kind: 'union',
        types: ['unknown', 'string'],
    });

    t.deepEqual(simplify(parse('never & string'), { simplifyTopTypesInIntersections: false }), {
        kind: 'intersection',
        types: ['never', 'string'],
    });
});

test('simplify normalize generic record', (t) => {
    // record<string, V> → record<V>
    t.deepEqual(simplify(parse('record<string, number>')), {
        kind: 'record',
        value: 'number',
    });
    // record<number, V> unchanged
    t.deepEqual(simplify(parse('record<number, boolean>')), {
        kind: 'record',
        key: 'number',
        value: 'boolean',
    });
});

test('simplify normalize generic array', (t) => {
    // array<any> → array
    t.is(simplify(parse('any[]')), 'array');
    t.is(simplify(parse('unknown[]')), 'array');
    // unchanged
    t.deepEqual(simplify(parse('number[]')), {
        kind: 'array',
        element: 'number',
    });
    t.deepEqual(simplify(parse('never[]')), {
        kind: 'array',
        element: 'never',
    });
});

test('simplify normalize options disabled', (t) => {
    t.deepEqual(simplify(parse('record<string, number>'), { normalizeGenericRecord: false }), {
        kind: 'record',
        key: 'string',
        value: 'number',
    });
    t.deepEqual(simplify(parse('array<any>'), { normalizeGenericArray: false }), {
        kind: 'array',
        element: 'any',
    });
});
