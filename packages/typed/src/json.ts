import type { JSONSchema7 } from 'json-schema';
import type { KnownType, NamedType, Type } from './parser.js';

/** Converts a KnownType or NamedType into JSON Schema */
function string(type: KnownType | NamedType): JSONSchema7 {
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
            return { type: 'array' };
        case 'record':
            return { type: 'object' };
        case 'extern':
            return {};
        case 'any':
            return {};
        case 'unknown':
            return {};
        case 'never':
            return {};
        default:
            return {};
    }
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type): JSONSchema7 {
    if (typeof type === 'string') {
        return string(type);
    }
    if (type.kind === 'array') {
        return {
            type: 'array',
            items: toJSONSchema(type.element),
        };
    }
    if (type.kind === 'union') {
        return {
            anyOf: type.types.map(toJSONSchema),
        };
    }
    if (type.kind === 'record') {
        const properties: Record<string, JSONSchema7> = {};
        const required: string[] = [];
        for (const field of type.fields) {
            properties[field.name] = toJSONSchema(field.type);
            if (!field.optional) {
                required.push(field.name);
            }
        }
        return {
            type: 'object',
            properties,
            required: required.length > 0 ? required : undefined,
        };
    }
    (type) satisfies never;
    return {};
}
