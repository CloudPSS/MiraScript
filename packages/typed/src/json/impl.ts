import type { JSONSchema } from 'json-schema-typed';
import type { Type } from '../parser.js';
import { simplify } from '../simplifier/index.js';
import type { ToJSONSchemaOptions } from './index.js';
import { string } from './string.js';
import { template } from './template.js';
import { tuple } from './tuple.js';
import { literal } from './literal.js';
import { record } from './record.js';
import { intersection, union } from './union-intersection.js';

/** Options for toJSONSchemaImpl */
export type ToJSONSchemaOptionsImpl = Required<ToJSONSchemaOptions>;

/** Converts a Type object into JSON Schema */
export function toJSONSchemaImpl(type: Type, options: ToJSONSchemaOptionsImpl): JSONSchema {
    const simplified = simplify(type);
    if (typeof simplified == 'symbol') {
        return true;
    }
    if (typeof simplified == 'string') {
        return string(simplified);
    }
    if (simplified.kind === 'array') {
        return {
            type: 'array',
            items: toJSONSchemaImpl(simplified.element, options),
        };
    }
    if (simplified.kind === 'union') {
        return union(simplified, options);
    }
    if (simplified.kind === 'intersection') {
        return intersection(simplified, options);
    }
    if (simplified.kind === 'record') {
        return record(simplified, options);
    }
    if (simplified.kind === 'literal') {
        return literal(simplified);
    }
    if (simplified.kind === 'function') {
        return false;
    }
    if (simplified.kind === 'template') {
        return template(simplified);
    }
    if (simplified.kind === 'tuple') {
        return tuple(simplified, options);
    }
    if (simplified.kind === 'reflection') {
        return true;
    }
    /* c8 ignore next 3 */
    simplified satisfies never;
    return true;
}
