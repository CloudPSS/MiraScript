import type { RecordField, RecordType } from '../parser.js';
import { type SimplifyImplOptions, simplifyImpl } from './impl.js';

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
    } else {
        if (record.key != null) record.key = simplifyImpl(record.key, options);
        record.value = simplifyImpl(record.value, options);
        if (options.normalizeGenericRecord && record.key === 'string') {
            delete record.key;
        }
    }
    return record;
}
