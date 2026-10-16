import { type JSONSchema, $schema } from 'json-schema-typed';
import type { Type } from '../parser.js';
import { toJSONSchemaImpl } from './impl.js';

export type { JSONSchema } from 'json-schema-typed';

/** Options for toJSONSchema */
export interface ToJSONSchemaOptions {
    /** When true, allow missing/extra object fields and omit tuple length constraints. */
    loose?: boolean;
    /** Include the JSON Schema dialect declaration. Defaults to true. */
    $schema?: boolean;
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(
    type: Type,
    options?: ToJSONSchemaOptions & { $schema?: true },
): JSONSchema.Interface & { $schema: typeof $schema };
/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type, options: ToJSONSchemaOptions): JSONSchema.Interface;
/** Converts a Type object into JSON Schema */
export function toJSONSchema(type: Type, options?: ToJSONSchemaOptions): JSONSchema.Interface {
    const schema = toJSONSchemaImpl(type, { loose: options?.loose ?? false });
    const result = (options?.$schema ?? true) ? { $schema } : {};
    Object.assign(schema, result);
    return schema;
}
