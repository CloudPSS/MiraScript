import type { GenericType, RecordField, RecordType, Type } from './parser.js';

/** Controls which type simplifications are applied. */
export interface SimplifyOptions {
    /** Flatten nested union nodes. */
    flattenUnions?: boolean;
    /** Flatten nested intersection nodes. */
    flattenIntersections?: boolean;
    /** Remove duplicate members inside union nodes. */
    deduplicateUnions?: boolean;
    /** Remove duplicate members inside intersection nodes. */
    deduplicateIntersections?: boolean;
    /** Remove single-member union nodes. */
    unwrapSingleUnion?: boolean;
    /** Remove single-member intersection nodes. */
    unwrapSingleIntersection?: boolean;
    /** Distribute intersections over unions. */
    distributeIntersectionsOverUnions?: boolean;
    /** Merge intersections of explicit record fields. */
    mergeRecordIntersections?: boolean;
}

const DEFAULT_OPTIONS: Required<SimplifyOptions> = {
    flattenUnions: true,
    flattenIntersections: true,
    deduplicateUnions: true,
    deduplicateIntersections: true,
    unwrapSingleUnion: true,
    unwrapSingleIntersection: true,
    distributeIntersectionsOverUnions: true,
    mergeRecordIntersections: true,
};

/** Fills in default simplification options. */
function normalizeOptions(options?: SimplifyOptions): Required<SimplifyOptions> {
    return { ...DEFAULT_OPTIONS, ...options };
}

/** Checks whether a type is represented by an object node. */
function isTypeObject(type: Type): type is Exclude<Type, GenericType | string> {
    return typeof type === 'object';
}

/** Checks whether a record type uses the explicit fields form. */
function isFieldRecordType(type: Type): type is Extract<RecordType, { fields: RecordField[] }> {
    return isTypeObject(type) && type.kind === 'record' && 'fields' in type;
}

/** Builds a stable key for type-level deduplication within one simplify call. */
function getTypeDedupKey(type: Type, symbols: Map<symbol, number>): string {
    if (typeof type === 'string') return `string:${type}`;
    if (typeof type === 'symbol') {
        const existing = symbols.get(type);
        if (existing != null) return `symbol:${existing}`;
        const next = symbols.size + 1;
        symbols.set(type, next);
        return `symbol:${next}`;
    }
    switch (type.kind) {
        case 'array':
            return `array:${getTypeDedupKey(type.element, symbols)}`;
        case 'union':
            return `union:[${type.types.map((t) => getTypeDedupKey(t, symbols)).join(',')}]`;
        case 'intersection':
            return `intersection:[${type.types.map((t) => getTypeDedupKey(t, symbols)).join(',')}]`;
        case 'record':
            if ('fields' in type) {
                return `recordFields:[${type.fields
                    .map((f) => `${f.name}:${String(Boolean(f.optional))}:${getTypeDedupKey(f.type, symbols)}`)
                    .join(',')}]`;
            }
            return `recordKV:${type.key == null ? 'none' : getTypeDedupKey(type.key, symbols)}:${getTypeDedupKey(type.value, symbols)}`;
        case 'literal':
            return `literal:${typeof type.value}:${String(type.value)}`;
        case 'template':
            return `template:[${type.parts.map((p) => getTypeDedupKey(p, symbols)).join(',')}]`;
        case 'function':
            return `function:${
                type.name ?? ''
            }:<${(type.typeParams ?? []).map((p) => getTypeDedupKey(p, symbols)).join(',')}>(${type.params
                .map((p) => `${p.name}:${String(Boolean(p.spread))}:${getTypeDedupKey(p.type, symbols)}`)
                .join(',')})=>${type.returns == null ? 'void' : getTypeDedupKey(type.returns, symbols)}`;
        default:
            return 'unknown';
    }
}

/** Removes duplicate members from union/intersection type member lists. */
function deduplicateTypeMembers(types: Type[]): Type[] {
    const symbols = new Map<symbol, number>();
    const seen = new Set<string>();
    const result: Type[] = [];
    for (const type of types) {
        const key = getTypeDedupKey(type, symbols);
        if (seen.has(key)) continue;
        seen.add(key);
        result.push(type);
    }
    return result;
}

/** Flattens nested union nodes when the corresponding option is enabled. */
function flattenUnionTypes(types: Type[], options: Required<SimplifyOptions>): Type[] {
    if (!options.flattenUnions) return types;
    const result: Type[] = [];
    for (const type of types) {
        if (isTypeObject(type) && type.kind === 'union') {
            result.push(...flattenUnionTypes(type.types, options));
        } else {
            result.push(type);
        }
    }
    return result;
}

/** Flattens nested intersection nodes when the corresponding option is enabled. */
function flattenIntersectionTypes(types: Type[], options: Required<SimplifyOptions>): Type[] {
    if (!options.flattenIntersections) return types;
    const result: Type[] = [];
    for (const type of types) {
        if (isTypeObject(type) && type.kind === 'intersection') {
            result.push(...flattenIntersectionTypes(type.types, options));
        } else {
            result.push(type);
        }
    }
    return result;
}

/** Simplifies a record field recursively. */
function simplifyRecordField(field: RecordField, options: Required<SimplifyOptions>): RecordField {
    return {
        ...field,
        type: simplifyImpl(field.type, options),
    };
}

/** Merges explicit record fields across an intersection. */
function mergeRecordFieldIntersections(types: Array<Extract<RecordType, { fields: RecordField[] }>>): Type {
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

/** Distributes intersections over unions using a cartesian product. */
function distributeIntersectionsOverUnions(types: Type[], options: Required<SimplifyOptions>): Type {
    let combinations: Type[][] = [[]];
    for (const type of types) {
        const choices = isTypeObject(type) && type.kind === 'union' ? type.types : [type];
        const next: Type[][] = [];
        for (const combo of combinations) {
            for (const choice of choices) {
                next.push([...combo, choice]);
            }
        }
        combinations = next;
    }

    const branches = combinations.map((combo) =>
        simplifyImpl(
            { kind: 'intersection', types: combo },
            {
                ...options,
                distributeIntersectionsOverUnions: false,
            },
        ),
    );
    if (branches.length === 1) return branches[0]!;
    return { kind: 'union', types: branches };
}

/** Simplifies a Type AST in place, optionally disabling individual normalization passes. */
function simplifyImpl(type: Type, config: Required<SimplifyOptions>): Type {
    if (typeof type === 'symbol' || typeof type === 'string') {
        return type;
    }

    if (type.kind === 'array') {
        type.element = simplifyImpl(type.element, config);
        return type;
    }

    if (type.kind === 'function') {
        for (const param of type.params) {
            param.type = simplifyImpl(param.type, config);
        }
        if (type.returns != null) type.returns = simplifyImpl(type.returns, config);
        return type;
    }

    if (type.kind === 'literal') {
        return type;
    }

    if (type.kind === 'template') {
        type.parts = type.parts.map((part) => simplifyImpl(part, config));
        return type;
    }

    if (type.kind === 'record') {
        if ('fields' in type) {
            type.fields = type.fields.map((field) => simplifyRecordField(field, config));
        } else {
            if (type.key != null) type.key = simplifyImpl(type.key, config);
            type.value = simplifyImpl(type.value, config);
        }
        return type;
    }

    if (type.kind === 'union') {
        let simplifiedTypes = flattenUnionTypes(
            type.types.map((item) => simplifyImpl(item, config)),
            config,
        );
        if (config.deduplicateUnions) {
            simplifiedTypes = deduplicateTypeMembers(simplifiedTypes);
        }
        if (config.unwrapSingleUnion && simplifiedTypes.length === 1) {
            return simplifiedTypes[0]!;
        }
        type.types = simplifiedTypes;
        return type;
    }

    if (type.kind === 'intersection') {
        let simplifiedTypes = flattenIntersectionTypes(
            type.types.map((item) => simplifyImpl(item, config)),
            config,
        );
        if (config.deduplicateIntersections) {
            simplifiedTypes = deduplicateTypeMembers(simplifiedTypes);
        }
        if (
            config.distributeIntersectionsOverUnions &&
            simplifiedTypes.some((item) => isTypeObject(item) && item.kind === 'union')
        ) {
            return distributeIntersectionsOverUnions(simplifiedTypes, config);
        }
        if (config.mergeRecordIntersections) {
            const recordTypes = simplifiedTypes.filter(isFieldRecordType);
            if (recordTypes.length >= 2) {
                const nonRecordTypes = simplifiedTypes.filter((item) => !isFieldRecordType(item));
                const mergedRecord = simplifyImpl(mergeRecordFieldIntersections(recordTypes), config);
                let mergedTypes = [mergedRecord, ...nonRecordTypes];
                if (config.deduplicateIntersections) {
                    mergedTypes = deduplicateTypeMembers(mergedTypes);
                }
                if (config.unwrapSingleIntersection && mergedTypes.length === 1) {
                    return mergedTypes[0]!;
                }
                type.types = mergedTypes;
                return type;
            }
        }
        if (config.unwrapSingleIntersection && simplifiedTypes.length === 1) {
            return simplifiedTypes[0]!;
        }
        type.types = simplifiedTypes;
        return type;
    }

    /* c8 ignore next 3 */
    (type) satisfies never;
    return type;
}

/** Simplifies a Type AST in place, optionally disabling individual normalization passes. */
export function simplify(type: Type, options?: SimplifyOptions): Type {
    const config = normalizeOptions(options);
    return simplifyImpl(type, config);
}
