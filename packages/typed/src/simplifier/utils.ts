import type { Type, GenericType, RecordField, RecordType } from '../parser.js';

/** Checks whether a type is represented by an object node. */
export function isTypeObject(type: Type): type is Exclude<Type, GenericType | string> {
    return typeof type === 'object';
}

/** Checks whether a record type uses the explicit fields form. */
export function isFieldRecordType(type: Type): type is Extract<RecordType, { fields: RecordField[] }> {
    return isTypeObject(type) && type.kind === 'record' && 'fields' in type;
}
