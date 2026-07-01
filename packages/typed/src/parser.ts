import { parse as _parse } from '../dist/type.js';

/** MiraScript type */
export type Type =
    | KnownType
    | NamedType
    | GenericType
    | LiteralType
    | TemplateType
    | ArrayType
    | UnionType
    | RecordType
    | FunctionType;

/** Generic type parameter reference in MiraScript */
export type GenericType = symbol;

/** Literal type in MiraScript */
export interface LiteralType {
    /** Tag */
    kind: 'literal';
    /** Literal value */
    value: string | boolean;
}

/** Template literal type in MiraScript */
export interface TemplateType {
    /** Tag */
    kind: 'template';
    /** Interpolated parts, strings are represented as {@link LiteralType} objects, other parts are nested {@link Type} objects. */
    parts: Type[];
}

/** Known type names in MiraScript */
export type KnownType =
    'string' | 'number' | 'boolean' | 'nil' | 'array' | 'record' | 'extern' | 'any' | 'unknown' | 'never';

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
export type RecordType =
    | {
          /** Tag */
          kind: 'record';
          /** Fields in the record */
          fields: RecordField[];
      }
    | {
          /** Tag */
          kind: 'record';
          /** Key type for record<K, V> */
          key?: Type;
          /** Value type for record<T> or record<K, V> */
          value: Type;
      };

/** Record field in MiraScript */
export interface RecordField {
    /** Field name */
    name: string;
    /** Whether the field is optional */
    optional?: boolean;
    /** Field type */
    type: Type;
}

/** Function type in MiraScript */
export interface FunctionType {
    /** Tag */
    kind: 'function';
    /** Function name (only allowed at the top level) */
    name?: string;
    /** Generic type parameters */
    typeParams?: GenericType[];
    /** Function parameters */
    params: FunctionParameter[];
    /** Return type */
    returns?: Type;
}

/** Function parameter in MiraScript */
export interface FunctionParameter {
    /** Parameter name */
    name: string;
    /** Parameter type */
    type: Type;
    /** Whether the parameter is a rest parameter */
    spread?: boolean;
}

/**
 * Replaces generic type parameter names in the AST with unique symbols,
 * scoped per function declaration.
 */
function resolveGenerics(type: Type, scope = new Map<string, GenericType>()): Type {
    /* c8 ignore next 3 */
    if (typeof type == 'symbol') {
        return type;
    }
    if (typeof type == 'string') {
        return scope.get(type) ?? type;
    }
    if (type.kind === 'function') {
        const newScope = new Map(scope);
        for (const sym of type.typeParams ?? []) {
            newScope.set(sym.description!, sym);
        }
        for (const param of type.params) {
            param.type = resolveGenerics(param.type, newScope);
        }
        if (type.returns != null) type.returns = resolveGenerics(type.returns, newScope);
        return type;
    }
    if (type.kind === 'array') {
        type.element = resolveGenerics(type.element, scope);
        return type;
    }
    if (type.kind === 'union') {
        type.types = type.types.map((t) => resolveGenerics(t, scope));
        return type;
    }
    if (type.kind === 'record') {
        if ('fields' in type) {
            for (const field of type.fields) {
                field.type = resolveGenerics(field.type, scope);
            }
        } else {
            if (type.key != null) type.key = resolveGenerics(type.key, scope);
            if (type.value != null) type.value = resolveGenerics(type.value, scope);
        }
        return type;
    }
    if (type.kind === 'literal') {
        return type;
    }
    if (type.kind === 'template') {
        type.parts = type.parts.map((part) => resolveGenerics(part, scope));
        return type;
    }
    /* c8 ignore next 3 */
    (type) satisfies never;
    return type;
}

/**
 * Parses a type string into a Type object.
 */
export function parse(type: string): Type {
    return resolveGenerics(_parse(type) as Type);
}
