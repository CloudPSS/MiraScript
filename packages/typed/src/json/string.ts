import type { JSONSchema } from 'json-schema-typed';
import type { KnownType, NamedType } from '../parser.js';
import { anySchema, neverSchema } from './boolean.js';

/** Converts a KnownType or NamedType into JSON Schema */
export function string(type: KnownType | NamedType): JSONSchema.Interface {
    switch (type) {
        case 'string':
            return { type: 'string' };
        case 'number':
            return { type: 'number' };
        case 'boolean':
            return { type: 'boolean' };
        case 'nil':
            return { type: 'null' };
        case 'array':
            return { type: 'array', items: anySchema };
        case 'record':
            return { type: 'object' };
        case 'extern':
            return anySchema;
        case 'any':
            return anySchema;
        case 'unknown':
            return anySchema;
        case 'never':
            return neverSchema;
        default:
            return anySchema;
    }
}
