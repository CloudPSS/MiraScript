import type { JSONSchema } from 'json-schema-typed';
import type { LiteralType, Type } from '../parser.js';

/** Converts a LiteralType into JSON Schema */
export function literal(type: LiteralType): JSONSchema {
    return { const: type.value };
}

/** Converts a union of LiteralTypes into a single JSON Schema enum */
export function literalEnum(types: LiteralType[]): JSONSchema {
    const schema: JSONSchema = {
        enum: types.map((t) => t.value),
    };
    return schema;
}

/** Type guard for LiteralType */
export function isLiteralType(type: Type): type is LiteralType {
    return typeof type == 'object' && type.kind === 'literal';
}
