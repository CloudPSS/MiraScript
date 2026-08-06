import type { JSONSchema } from 'json-schema-typed';
import type { IntersectionType, LiteralType, UnionType } from '../parser.js';
import { type ToJSONSchemaOptionsImpl, toJSONSchemaImpl } from './impl.js';
import { isLiteralType, literalEnum } from './literal.js';

/** Converts a union type to JSON representation */
export function union(union: UnionType, options: ToJSONSchemaOptionsImpl): JSONSchema {
    const anyOf: JSONSchema[] = [];
    const literals: LiteralType[] = [];
    for (const t of union.types) {
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

/** Converts an intersection type to JSON representation */
export function intersection(intersection: IntersectionType, options: ToJSONSchemaOptionsImpl): JSONSchema {
    const allOf = intersection.types.map((t) => toJSONSchemaImpl(t, options));
    if (allOf.length === 1) {
        return allOf[0]!;
    }
    return { allOf };
}
