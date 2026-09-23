import type { RecordField, RecordType } from '../parser.js';
import { type SimplifyImplOptions, simplifyImpl } from './impl.js';
import { isFieldRecordType } from './utils.js';

/** Simplifies a record field recursively. */
function simplifyRecordField(field: RecordField, options: SimplifyImplOptions): RecordField {
    return {
        ...field,
        type: simplifyImpl(field.type, options),
    };
}

/** Simplifies a record recursively. */
export function simplifyRecord(record: RecordType, options: SimplifyImplOptions): RecordType {
    if ('fields' in record) {
        record.fields = record.fields.map((field) => simplifyRecordField(field, options));
        if (record.rest == null) return record;
        const rest = simplifyImpl(record.rest, options);
        if (!options.expandRecordSpreads || !isFieldRecordType(rest)) {
            record.rest = rest;
            return record;
        }
        // Inline record spread: ..(a: T) → a: T
        // Explicit fields take precedence over fields spread from the rest type
        const declared = new Set(record.fields.map((field) => field.name));
        for (const field of rest.fields) {
            if (declared.has(field.name)) continue;
            declared.add(field.name);
            record.fields.push(field);
        }
        // A nested rest type is kept as the rest of this record
        if (rest.rest == null) {
            delete record.rest;
        } else {
            record.rest = rest.rest;
        }
    } else {
        if (record.key != null) record.key = simplifyImpl(record.key, options);
        record.value = simplifyImpl(record.value, options);
        if (options.normalizeGenericRecord && record.key === 'string') {
            delete record.key;
        }
    }
    return record;
}
