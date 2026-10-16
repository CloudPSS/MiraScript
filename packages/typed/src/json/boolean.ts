import type { JSONSchema } from 'json-schema-typed';

/** Object forms of boolean schemas, used when an object schema is required. */
export const anySchema: JSONSchema.Interface = {};
export const neverSchema: JSONSchema.Interface = { not: {} };

/** Prefer boolean schemas in positions that accept them. */
export function compactSchema(schema: JSONSchema.Interface): JSONSchema {
    if (schema === anySchema) return true;
    if (schema === neverSchema) return false;
    return schema;
}
