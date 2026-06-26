import type { JSONSchema7 } from 'json-schema';
import { REG_NUMBER } from '@mirascript/constants';
import type { KnownType, LiteralType, NamedType, TemplateType, Type } from './parser.js';

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

/** Escapes a literal string for use in a regular expression */
function escapeRegex(value: string): string {
    return value.replaceAll(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`);
}

const RE_ANY = '.*?';
const RE_NUMBER = REG_NUMBER.source;
const RE_BOOLEAN = 'true|false';

/** Converts a template interpolation part into a regex pattern fragment */
function templatePartPattern(part: Type, grouping: boolean): string {
    let result: string;
    if (typeof part === 'symbol') {
        result = RE_ANY;
    } else if (typeof part === 'string') {
        switch (part) {
            case 'number':
                result = RE_NUMBER;
                break;
            case 'boolean':
                result = RE_BOOLEAN;
                break;
            case 'nil':
            case 'never':
                result = '';
                break;
            case 'array':
            case 'record':
            case 'extern':
            case 'any':
            case 'unknown':
            case 'string':
            default:
                result = RE_ANY;
                break;
        }
    } else if (part.kind === 'literal') {
        if (typeof part.value === 'boolean') {
            result = String(part.value);
        } else {
            result = escapeRegex(part.value);
        }
    } else if (part.kind === 'union') {
        const patterns = new Set(part.types.map((p) => templatePartPattern(p, false)));
        if (patterns.has(RE_ANY)) {
            result = RE_ANY;
        } else {
            const hasEmpty = patterns.delete('');
            result = Array.from(patterns).join('|');
            if (hasEmpty) {
                result = `(${result})?`;
            }
        }
    } else {
        result = RE_ANY;
    }
    if (!grouping) return result;
    if (result.startsWith('(')) return result;
    return `(${result})`;
}

/** Converts a TemplateType into a JSON Schema pattern */
function templatePattern(type: TemplateType): string {
    return `^${type.parts
        .map((p) => {
            if (typeof p == 'object' && p.kind === 'literal' && typeof p.value === 'string') {
                return templatePartPattern(p, false);
            }
            return templatePartPattern(p, true);
        })
        .join('')}$`;
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type): JSONSchema7 {
    if (typeof type === 'symbol') {
        return {};
    }
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
        const anyOf = [];
        const literals = [];
        for (const t of type.types) {
            if (isLiteralType(t)) {
                literals.push(t);
            } else {
                anyOf.push(toJSONSchema(t));
            }
        }
        if (literals.length > 0) {
            anyOf.push(literalEnum(literals));
        }
        if (anyOf.length === 1) {
            return anyOf[0]!;
        }
        return { anyOf };
    }
    if (type.kind === 'record') {
        if (type.value !== undefined) {
            return {
                type: 'object',
                additionalProperties: toJSONSchema(type.value),
            };
        }
        const properties: Record<string, JSONSchema7> = {};
        const required: string[] = [];
        for (const field of type.fields) {
            properties[field.name] = toJSONSchema(field.type);
            if (!field.optional) {
                required.push(field.name);
            }
        }
        const schema: JSONSchema7 = {
            type: 'object',
            properties,
        };
        if (required.length > 0) {
            schema.required = required;
        }
        return schema;
    }
    if (type.kind === 'literal') {
        return literal(type);
    }
    if (type.kind === 'function') {
        return {};
    }
    if (type.kind === 'template') {
        return { type: 'string', pattern: templatePattern(type) };
    }
    /* c8 ignore next 3 */
    (type) satisfies never;
    return {};
}

/** Converts a LiteralType into JSON Schema */
function literal(type: LiteralType): JSONSchema7 {
    if (typeof type.value === 'boolean') {
        return { type: 'boolean', const: type.value };
    }
    return { type: 'string', const: type.value };
}

/** Converts a union of LiteralTypes into a single JSON Schema enum */
function literalEnum(types: LiteralType[]): JSONSchema7 {
    const values = types.map((t) => t.value);
    const valueTypes = new Set(values.map((v) => (typeof v === 'boolean' ? 'boolean' : 'string')));
    const schema: JSONSchema7 = { enum: values };
    if (valueTypes.size === 1) {
        schema.type = valueTypes.values().next().value!;
    }
    return schema;
}

/** Type guard for LiteralType */
function isLiteralType(type: Type): type is LiteralType {
    return typeof type === 'object' && type.kind === 'literal';
}
