import { type JSONSchema, $schema } from 'json-schema-typed';
import type { Type } from '../parser.js';
import { toJSONSchemaImpl } from './impl.js';

export type { JSONSchema } from 'json-schema-typed';

/** Options for toJSONSchema */
export interface ToJSONSchemaOptions {
    /** When true, object schemas allow arbitrary additional properties */
    loose?: boolean;
}

/** Converts a Type object into JSON Schema */
export function toJSONSchema(
    type: Type,
    options?: ToJSONSchemaOptions,
): JSONSchema.Interface & { $schema: typeof $schema } {
    let schema = toJSONSchemaImpl(type, { loose: options?.loose ?? false });
    if (schema === true) {
        schema = {};
    } else if (schema === false) {
        schema = { not: true };
    }
    return {
        ...schema,
        $schema,
    };
}
