import type { RecordType, RecordField, Type } from '../parser.js';

/** Merges explicit record fields across an intersection. */
export function mergeRecordFieldIntersections(types: Array<Extract<RecordType, { fields: RecordField[] }>>): Type {
    const merged = new Map<string, { optional: boolean; type: Type }>();
    for (const record of types) {
        for (const field of record.fields) {
            const prev = merged.get(field.name);
            if (prev == null) {
                merged.set(field.name, {
                    optional: field.optional ?? false,
                    type: field.type,
                });
                continue;
            }
            merged.set(field.name, {
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
