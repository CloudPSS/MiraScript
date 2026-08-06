import type { JSONSchema } from 'json-schema-typed';
import { simplify } from '../simplifier.js';
import type { LiteralType, Type } from '../parser.js';
import { string } from './string.js';
import { template } from './template.js';
import { tuple } from './tuple.js';
import type { ToJSONSchemaOptions } from './index.js';
import { isLiteralType, literalEnum, literal } from './literal.js';
import { record } from './record.js';

/** Options for toJSONSchemaImpl */
export type ToJSONSchemaOptionsImpl = Required<ToJSONSchemaOptions>;

/** Converts a Type object into JSON Schema */
export function toJSONSchemaImpl(type: Type, options: ToJSONSchemaOptionsImpl): JSONSchema {
    const simplified = simplify(type);
    if (typeof simplified == 'symbol') {
        return {};
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
        const anyOf: JSONSchema[] = [];
        const literals: LiteralType[] = [];
        for (const t of simplified.types) {
            if (isLiteralType(t)) {
                literals.push(t);
                continue;
            }
            const child = toJSONSchemaImpl(t, options);
            if (child === true) {
                return true;
            }
            if (child === false) {
                continue;
            }
            anyOf.push(child);
        }
        if (literals.length > 0) {
            anyOf.push(literalEnum(literals));
        }
        if (anyOf.length === 0) {
            return false;
        }
        if (anyOf.length === 1) {
            return anyOf[0]!;
        }
        return { anyOf };
    }
    if (simplified.kind === 'intersection') {
        const allOf = simplified.types.map((t) => toJSONSchemaImpl(t, options));
        if (allOf.length === 1) {
            return allOf[0]!;
        }
        return { allOf };
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
