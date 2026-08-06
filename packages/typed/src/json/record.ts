import type { JSONSchema } from 'json-schema-typed';
import type { RecordField, RecordType, Type } from '../parser.js';
import { toJSONSchemaImpl, type ToJSONSchemaOptionsImpl } from './impl.js';
import { templatePartPattern, RE_ANY } from './template.js';
import { isLiteralType } from './literal.js';

/** Convert fields record into JSON Schema */
function fieldsRecord(fields: readonly RecordField[], options: ToJSONSchemaOptionsImpl): JSONSchema {
    const properties: Record<string, JSONSchema> = {};
    const required: string[] = [];
    for (const field of fields) {
        properties[field.name] = toJSONSchemaImpl(field.type, options);
        if (!options.loose && !field.optional) {
            required.push(field.name);
        }
    }
    const schema: JSONSchema = {
        type: 'object',
        properties,
        additionalProperties: options.loose,
    };
    if (required.length > 0) {
        schema.required = required;
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
        return fieldsRecord(simplified.fields, options);
    }
    return genericRecord(simplified.key ?? 'string', simplified.value, options);
}
