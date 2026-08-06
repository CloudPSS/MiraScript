import type { Type, GenericType, RecordField, RecordType, UnionType, IntersectionType } from '../parser.js';

/** Checks whether a type is represented by an object node. */
export function isTypeObject(type: Type): type is Exclude<Type, GenericType | string> {
    return typeof type === 'object';
}

/** Checks whether a type is union. */
export function isUnionType(type: Type): type is UnionType {
    return isTypeObject(type) && type.kind === 'union';
}

/** Checks whether a type is intersection. */
export function isIntersectionType(type: Type): type is IntersectionType {
    return isTypeObject(type) && type.kind === 'intersection';
}

/** Checks whether a record type uses the explicit fields form. */
export function isFieldRecordType(type: Type): type is Extract<RecordType, { fields: RecordField[] }> {
    return isTypeObject(type) && type.kind === 'record' && 'fields' in type;
}
