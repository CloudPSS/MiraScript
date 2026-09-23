import type { JSONSchema } from 'json-schema-typed';
import type { RecordField, RecordType, Type } from '../parser.js';
import { toJSONSchemaImpl, type ToJSONSchemaOptionsImpl } from './impl.js';
import { templatePartPattern, RE_ANY } from './template.js';
import { isLiteralType } from './literal.js';

/**
 * Merges the JSON Schema of a record rest type (`..restType`) into the schema being built,
 * and returns the additional properties schema for the remaining fields.
 */
function restType(
    rest: Type,
    properties: Map<string, JSONSchema>,
    required: string[],
    patternProperties: Map<string, JSONSchema>,
    options: ToJSONSchemaOptionsImpl,
): JSONSchema {
    const schema = toJSONSchemaImpl(rest, options);
    // Rest types that are not records (e.g. `..string`, `..MyType`) do not constrain the rest fields,
    // consistent with tuple spreads where non-array rest elements are treated as `any`.
    if (typeof schema === 'boolean' || schema.type !== 'object') return true;
    for (const [name, property] of Object.entries(schema.properties ?? {})) {
        // Explicit fields take precedence over the fields spread from the rest type.
        if (properties.has(name)) continue;
        properties.set(name, property);
        if (!options.loose && !required.includes(name) && schema.required?.includes(name)) {
            required.push(name);
        }
    }
    for (const [pattern, property] of Object.entries(schema.patternProperties ?? {})) {
        patternProperties.set(pattern, property);
    }
    return schema.additionalProperties ?? true;
}

/** Keep rest patterns from constraining fields already named in the record. */
function excludeNamedProperties(pattern: string, names: string[]): string {
    const matches = names.filter((name) => new RegExp(pattern).test(name));
    if (matches.length === 0) return pattern;
    const escaped = matches.map((name) => name.replaceAll(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`));
    const guard = `(?!(?:${escaped.join('|')})$)`;
    return pattern.startsWith('^') ? `^${guard}${pattern.slice(1)}` : `^${guard}(?:${pattern})`;
}

/** Convert fields record into JSON Schema */
function fieldsRecord(
    fields: readonly RecordField[],
    rest: Type | undefined,
    options: ToJSONSchemaOptionsImpl,
): JSONSchema {
    if (typeof rest === 'object') {
        if (rest.kind === 'union') {
            return { anyOf: rest.types.map((branch) => fieldsRecord(fields, branch, options)) };
        }
        if (rest.kind === 'intersection') {
            return { allOf: rest.types.map((branch) => fieldsRecord(fields, branch, options)) };
        }
    }
    const properties = new Map<string, JSONSchema>();
    const required: string[] = [];
    const patternProperties = new Map<string, JSONSchema>();
    let additionalProperties: JSONSchema = options.loose;
    for (const field of fields) {
        properties.set(field.name, toJSONSchemaImpl(field.type, options));
        if (!options.loose && !field.optional && !required.includes(field.name)) {
            required.push(field.name);
        }
    }
    if (rest != null) {
        additionalProperties = restType(rest, properties, required, patternProperties, options);
    }
    const schema: JSONSchema = {
        type: 'object',
        properties: Object.fromEntries(properties),
        additionalProperties,
    };
    if (required.length > 0) {
        schema.required = required;
    }
    if (patternProperties.size > 0) {
        schema.patternProperties = Object.fromEntries(
            Array.from(patternProperties, ([pattern, value]) => [
                excludeNamedProperties(pattern, Array.from(properties.keys())),
                value,
            ]),
        );
    }
    return schema;
}

/** Convert generic record into JSON Schema */
function genericRecord(key: Type, value: Type, options: ToJSONSchemaOptionsImpl): JSONSchema {
    const valueSchema = toJSONSchemaImpl(value, options);

    if (typeof key == 'object') {
        if (isLiteralType(key)) {
            const schema: JSONSchema = {
                type: 'object',
                properties: { [String(key.value)]: valueSchema },
                additionalProperties: options.loose,
            };
            return schema;
        } else if (key.kind === 'union' && key.types.every(isLiteralType)) {
            const schema: JSONSchema = {
                type: 'object',
                properties: Object.fromEntries(key.types.map((t) => [String(t.value), valueSchema])),
                additionalProperties: options.loose,
            };
            return schema;
        }
    }
    const pattern = templatePartPattern(key, false);
    if (pattern === RE_ANY) {
        return {
            type: 'object',
            additionalProperties: valueSchema,
        };
    }
    const schema: JSONSchema = {
        type: 'object',
        patternProperties: { [`^${pattern}$`]: valueSchema },
        additionalProperties: options.loose,
    };
    return schema;
}

/** Converts a RecordType into JSON Schema */
export function record(simplified: RecordType, options: ToJSONSchemaOptionsImpl): JSONSchema {
    if ('fields' in simplified) {
        return fieldsRecord(simplified.fields, simplified.rest, options);
    }
    return genericRecord(simplified.key ?? 'string', simplified.value, options);
}
