import type { JSONSchema } from 'json-schema-typed';
import type { TupleType, Type } from '../parser.js';
import { deduplicateTypeMembers } from '../simplifier/dedup.js';
import { toJSONSchemaImpl, type ToJSONSchemaOptionsImpl } from './impl.js';
import { compactSchema } from './boolean.js';

/**
 * Converts a TupleType into JSON Schema
 */
export function tuple(tuple: TupleType, options: ToJSONSchemaOptionsImpl): JSONSchema.Interface {
    // Unwrap array rest elements (..T[] → T); non-array rest is treated as any.
    const elementTypes = tuple.elements.map<Type>((e) => {
        const t = e.type;
        if (!e.spread) return t;
        if (typeof t == 'object' && t.kind === 'array') {
            return t.element;
        }
        return 'any';
    });

    const schema: JSONSchema.Interface = { type: 'array' };
    const hasRest = tuple.elements.some((element) => element.spread === true);
    if (!options.loose) {
        schema.minItems = tuple.elements.filter((element) => !element.spread).length;
        if (!hasRest) schema.maxItems = tuple.elements.length;
    }

    // In loose mode, a fixed tuple still allows arbitrary extra elements.
    const canUseItems = (hasRest || !options.loose) && elementTypes.length > 0;
    if (canUseItems && deduplicateTypeMembers(elementTypes).length === 1) {
        schema.items = compactSchema(toJSONSchemaImpl(elementTypes[0]!, options));
        return schema;
    }

    const elementSchemas = elementTypes.map((type) => compactSchema(toJSONSchemaImpl(type, options)));
    // Different types such as any/unknown can still produce the same boolean schema.
    const first = elementSchemas[0];
    if (canUseItems && typeof first === 'boolean' && elementSchemas.every((element) => element === first)) {
        schema.items = first;
        return schema;
    }

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
