import type { JSONSchema } from 'json-schema-typed';
import { REG_NUMBER } from '@mirascript/constants';
import { simplify } from './simplifier.js';
import type { KnownType, LiteralType, NamedType, TemplateType, Type } from './parser.js';

/** Converts a KnownType or NamedType into JSON Schema */
function string(type: KnownType | NamedType): JSONSchema {
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
            return { type: 'array', items: {} };
        case 'record':
            return { type: 'object' };
        case 'extern':
            return {};
        case 'any':
            return {};
        case 'unknown':
            return {};
        case 'never':
            return false;
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
    } else if (part.kind === 'intersection') {
        // Regex intersection is not representable in general; keep behavior conservative.
        result = RE_ANY;
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

/** Converts a LiteralType into JSON Schema */
function literal(type: LiteralType): JSONSchema {
    return { const: type.value };
}

/** Converts a union of LiteralTypes into a single JSON Schema enum */
function literalEnum(types: LiteralType[]): JSONSchema {
    const schema: JSONSchema = {
        enum: types.map((t) => t.value),
    };
    return schema;
}

/** Type guard for LiteralType */
function isLiteralType(type: Type): type is LiteralType {
    return typeof type == 'object' && type.kind === 'literal';
}

/** Options for toJSONSchema */
export interface ToJSONSchemaOptions {
    /** When true, object schemas allow arbitrary additional properties */
    loose?: boolean;
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type, options?: ToJSONSchemaOptions): JSONSchema {
    const loose = options?.loose ?? false;
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
            items: toJSONSchema(simplified.element, options),
        };
    }
    if (simplified.kind === 'union') {
        const anyOf: JSONSchema[] = [];
        const literals: LiteralType[] = [];
        for (const t of simplified.types) {
            if (isLiteralType(t)) {
                literals.push(t);
            } else {
                anyOf.push(toJSONSchema(t, options));
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
    if (simplified.kind === 'intersection') {
        const allOf = simplified.types.map((t) => toJSONSchema(t, options));
        if (allOf.length === 1) {
            return allOf[0]!;
        }
        return { allOf };
    }
    if (simplified.kind === 'record') {
        if ('fields' in simplified) {
            const properties: Record<string, JSONSchema> = {};
            const required: string[] = [];
            for (const field of simplified.fields) {
                properties[field.name] = toJSONSchema(field.type, options);
                if (!field.optional && !loose) {
                    required.push(field.name);
                }
            }
            const schema: JSONSchema = {
                type: 'object',
                properties,
                additionalProperties: loose,
            };
            if (required.length > 0) {
                schema.required = required;
            }
            return schema;
        }

        const valueSchema = toJSONSchema(simplified.value, options);
        if (simplified.key == null) {
            return {
                type: 'object',
                additionalProperties: valueSchema,
            };
        }
        if (typeof simplified.key == 'object') {
            if (isLiteralType(simplified.key)) {
                const schema: JSONSchema = {
                    type: 'object',
                    properties: { [String(simplified.key.value)]: valueSchema },
                    additionalProperties: loose,
                };
                return schema;
            } else if (simplified.key.kind === 'union' && simplified.key.types.every(isLiteralType)) {
                const schema: JSONSchema = {
                    type: 'object',
                    properties: Object.fromEntries(simplified.key.types.map((t) => [String(t.value), valueSchema])),
                    additionalProperties: loose,
                };
                return schema;
            }
        }
        const pattern = templatePartPattern(simplified.key, false);
        if (pattern === RE_ANY) {
            return {
                type: 'object',
                additionalProperties: valueSchema,
            };
        }
        const schema: JSONSchema = {
            type: 'object',
            patternProperties: { [`^${pattern}$`]: valueSchema },
            additionalProperties: loose,
        };
        return schema;
    }
    if (simplified.kind === 'literal') {
        return literal(simplified);
    }
    if (simplified.kind === 'function') {
        return {};
    }
    if (simplified.kind === 'template') {
        return { type: 'string', pattern: templatePattern(simplified) };
    }
    /* c8 ignore next 3 */
    simplified satisfies never;
    return {};
}
