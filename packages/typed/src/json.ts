import type { JSONSchema } from 'json-schema-typed';
import { REG_NUMBER } from '@mirascript/constants';
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
    if (typeof type.value === 'boolean') {
        return { type: 'boolean', const: type.value };
    }
    return { type: 'string', const: type.value };
}

/** Converts a union of LiteralTypes into a single JSON Schema enum */
function literalEnum(types: LiteralType[]): JSONSchema {
    const values = types.map((t) => t.value);
    const valueTypes = new Set(values.map((v) => (typeof v === 'boolean' ? 'boolean' : 'string')));
    const schema: JSONSchema = { enum: values };
    if (valueTypes.size === 1) {
        schema.type = valueTypes.values().next().value!;
    }
    return schema;
}

/** Type guard for LiteralType */
function isLiteralType(type: Type): type is LiteralType {
    return typeof type === 'object' && type.kind === 'literal';
}

/** Type guard for object record with explicit fields */
function isFieldRecordType(type: Type): type is Extract<Type, { kind: 'record'; fields: unknown[] }> {
    return typeof type == 'object' && type.kind === 'record' && 'fields' in type;
}

/** Merges record field lists for intersection semantics. */
function mergeRecordFieldIntersections(types: Type[]): Type {
    const merged = new Map<string, { optional: boolean; type: Type }>();
    for (const t of types) {
        if (!isFieldRecordType(t)) continue;
        for (const field of t.fields) {
            const prev = merged.get(field.name);
            if (prev == null) {
                merged.set(field.name, {
                    optional: field.optional ?? false,
                    type: field.type,
                });
                continue;
            }
            merged.set(field.name, {
                // In intersections, required wins over optional.
                optional: (prev.optional ?? false) && (field.optional ?? false),
                type: {
                    kind: 'intersection',
                    types: [prev.type, field.type],
                },
            });
        }
    }

    return {
        kind: 'record',
        fields: Array.from(merged.entries()).map(([name, field]) => ({
            name,
            optional: field.optional,
            type: field.type,
        })),
    };
}

/** Flattens nested intersection nodes into a single-level list. */
function flattenIntersectionTypes(types: Type[]): Type[] {
    const result: Type[] = [];
    for (const t of types) {
        if (typeof t === 'object' && t.kind === 'intersection') {
            result.push(...flattenIntersectionTypes(t.types));
        } else {
            result.push(t);
        }
    }
    return result;
}

/** Distributes intersection over direct union members. */
function distributeIntersectionOverUnions(types: Type[]): Type[] {
    let combinations: Type[][] = [[]];
    for (const t of types) {
        const choices = typeof t === 'object' && t.kind === 'union' ? t.types : [t];
        const next: Type[][] = [];
        for (const combo of combinations) {
            for (const choice of choices) {
                next.push([...combo, choice]);
            }
        }
        combinations = next;
    }

    return combinations.map((combo) => {
        const flattened = flattenIntersectionTypes(combo);
        if (flattened.length === 1) return flattened[0]!;
        return { kind: 'intersection', types: flattened };
    });
}

/** Options for toJSONSchema */
export interface ToJSONSchemaOptions {
    /** When true, object schemas allow arbitrary additional properties */
    loose?: boolean;
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type, options?: ToJSONSchemaOptions): JSONSchema {
    const loose = options?.loose ?? false;
    if (typeof type === 'symbol') {
        return {};
    }
    if (typeof type === 'string') {
        return string(type);
    }
    if (type.kind === 'array') {
        return {
            type: 'array',
            items: toJSONSchema(type.element, options),
        };
    }
    if (type.kind === 'union') {
        const anyOf: JSONSchema[] = [];
        const literals = [];
        for (const t of type.types) {
            if (isLiteralType(t)) {
                literals.push(t);
            } else {
                const item = toJSONSchema(t, options);
                if (
                    typeof item == 'object' &&
                    item != null &&
                    'anyOf' in item &&
                    Object.keys(item).length === 1 &&
                    Array.isArray(item.anyOf)
                ) {
                    anyOf.push(...item.anyOf);
                } else {
                    anyOf.push(item);
                }
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
    if (type.kind === 'intersection') {
        const flattenedTypes = flattenIntersectionTypes(type.types);
        if (flattenedTypes.some((t) => typeof t == 'object' && t.kind === 'union')) {
            const distributed = distributeIntersectionOverUnions(flattenedTypes);
            if (distributed.length === 1) {
                return toJSONSchema(distributed[0]!, options);
            }
            return toJSONSchema({ kind: 'union', types: distributed }, options);
        }
        if (flattenedTypes.every(isFieldRecordType)) {
            const merged = mergeRecordFieldIntersections(flattenedTypes);
            return toJSONSchema(merged, options);
        }
        const allOf = flattenedTypes.map((t) => toJSONSchema(t, options));
        if (allOf.length === 1) {
            return allOf[0]!;
        }
        return { allOf };
    }
    if (type.kind === 'record') {
        if ('fields' in type) {
            const properties: Record<string, JSONSchema> = {};
            const required: string[] = [];
            for (const field of type.fields) {
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

        const valueSchema = toJSONSchema(type.value, options);
        if (type.key == null) {
            return {
                type: 'object',
                additionalProperties: valueSchema,
            };
        }
        if (typeof type.key == 'object') {
            if (isLiteralType(type.key)) {
                const schema: JSONSchema = {
                    type: 'object',
                    properties: { [String(type.key.value)]: valueSchema },
                    additionalProperties: loose,
                };
                return schema;
            } else if (type.key.kind === 'union' && type.key.types.every(isLiteralType)) {
                const schema: JSONSchema = {
                    type: 'object',
                    properties: Object.fromEntries(type.key.types.map((t) => [String(t.value), valueSchema])),
                    additionalProperties: loose,
                };
                return schema;
            }
        }
        const pattern = templatePartPattern(type.key, false);
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
