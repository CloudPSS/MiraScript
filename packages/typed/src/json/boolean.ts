import type { JSONSchema } from 'json-schema-typed';

/** Object forms of the JSON Schema boolean schemas, for clients that do not accept boolean schemas. */
export const anySchema: JSONSchema.Interface = {};
export const neverSchema: JSONSchema.Interface = { not: {} };

/** Converts an internal boolean option into its object-form JSON Schema equivalent. */
export function booleanSchema(value: boolean): JSONSchema.Interface {
    return value ? anySchema : neverSchema;
}
