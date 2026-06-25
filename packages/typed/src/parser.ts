import { parse as _parse } from '../dist/type.js';

/** MiraScript type */
export type Type = KnownType | NamedType | ArrayType | UnionType | RecordType;

/** Known type names in MiraScript */
export type KnownType =
    | 'string'
    | 'number'
    | 'boolean'
    | 'nil'
    | 'array'
    | 'record'
    | 'extern'
    | 'any'
    | 'unknown'
    | 'never';

/** Named type in MiraScript */
export type NamedType = string & Record<never, never>;

/** Union type in MiraScript */
export interface UnionType {
    /** Tag */
    kind: 'union';
    /** Types in the union */
    types: Type[];
}

/** Array type in MiraScript */
export interface ArrayType {
    /** Tag */
    kind: 'array';
    /** Element type */
    element: Type;
}

/** Record type in MiraScript */
export interface RecordType {
    /** Tag */
    kind: 'record';
    /** Fields in the record */
    fields: RecordField[];
}

/** Record field in MiraScript */
export interface RecordField {
    /** Field name */
    name: string;
    /** Whether the field is optional */
    optional?: boolean;
    /** Field type */
    type: Type;
}

/**
 * Parses a type string into a Type object.
 */
export function parse(type: string): Type {
    return _parse(type) as Type;
}
