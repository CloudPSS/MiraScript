import type { JSONSchema } from 'json-schema-typed';
import type { KnownType, NamedType } from '../parser.js';

/** Converts a KnownType or NamedType into JSON Schema */
export function string(type: KnownType | NamedType): JSONSchema {
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
            return { type: 'array', items: true };
        case 'record':
            return { type: 'object' };
        case 'extern':
            return true;
        case 'any':
            return true;
        case 'unknown':
            return true;
        case 'never':
            return false;
        default:
            return true;
    }
}
