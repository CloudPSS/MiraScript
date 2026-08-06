import type { JSONSchema } from 'json-schema-typed';
import type { TupleType } from '../parser.js';
import { toJSONSchemaImpl, type ToJSONSchemaOptionsImpl } from './impl.js';

/**
 * Converts a TupleType into JSON Schema
 */
export function tuple(tuple: TupleType, options: ToJSONSchemaOptionsImpl): JSONSchema {
    // Collect element schemas, unwrapping array from rest elements (..T[] → T).
    // Non-array rest (e.g. ..string) is treated as any ({}).
    const elementSchemas = tuple.elements.map((e) => {
        const t = e.type;
        if (!e.spread) return toJSONSchemaImpl(t, options);
        if (typeof t == 'object' && t.kind === 'array') {
            return toJSONSchemaImpl(t.element, options);
        }
        return true;
    });

    const prefixItems = [];
    let i = 0;
    for (; i < elementSchemas.length; i++) {
        const el = tuple.elements[i]!;
        if (el.spread) break;
        prefixItems.push(elementSchemas[i]!);
    }

    const items = [];
    let hasAny = false;
    for (; i < elementSchemas.length; i++) {
        const schema = elementSchemas[i]!;
        items.push(schema);
        if (schema === true) {
            hasAny = true;
        }
    }

    const schema: JSONSchema = { type: 'array' };
    if (prefixItems.length > 0) {
        schema.prefixItems = prefixItems;
    }
    if (hasAny) {
        schema.items = true;
    } else if (items.length > 1) {
        schema.items = { anyOf: items };
    } else if (items.length === 1) {
        schema.items = items[0]!;
    } else {
        schema.items = options.loose;
    }
    return schema;
}
