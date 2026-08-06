import type { GenericType, RecordField, RecordType, Type } from '../parser.js';
import { deduplicateTypeMembers } from './dedup.js';
import type { SimplifyOptions } from './index.js';

/** Top types that can be absorbed / eliminated in simplification. */
export type TopType = 'unknown' | 'never' | 'any';

/** Resolves the top types option into an array of top types. */
function resolveTopTypes(value: boolean | TopType[] | undefined): TopType[] {
    if (value === false || value == null) return [];
    if (value === true) return ['unknown', 'never', 'any'];
    return value;
}

/** Checks whether a type is represented by an object node. */
function isTypeObject(type: Type): type is Exclude<Type, GenericType | string> {
    return typeof type === 'object';
}

/** Checks whether a record type uses the explicit fields form. */
function isFieldRecordType(type: Type): type is Extract<RecordType, { fields: RecordField[] }> {
    return isTypeObject(type) && type.kind === 'record' && 'fields' in type;
}

/** Flattens nested union nodes when the corresponding option is enabled. */
function flattenUnionTypes(types: Type[], options: SimplifyImplOptions): Type[] {
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
function flattenIntersectionTypes(types: Type[], options: SimplifyImplOptions): Type[] {
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
function simplifyRecordField(field: RecordField, options: SimplifyImplOptions): RecordField {
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
function distributeIntersectionsOverUnions(types: Type[], options: SimplifyImplOptions): Type {
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

/** Options for simplifyImpl */
export type SimplifyImplOptions = Required<SimplifyOptions>;

/** Simplifies a Type AST in place, optionally disabling individual normalization passes. */
export function simplifyImpl(type: Type, config: SimplifyImplOptions): Type {
    if (typeof type === 'symbol' || typeof type === 'string') {
        return type;
    }

    if (type.kind === 'array') {
        type.element = simplifyImpl(type.element, config);
        if (config.normalizeGenericArray && (type.element === 'any' || type.element === 'unknown')) {
            return 'array';
        }
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

    if (type.kind === 'tuple') {
        type.elements = type.elements.flatMap((element) => {
            const simplifiedType = simplifyImpl(element.type, config);
            if (
                config.expandTupleSpreads &&
                element.spread &&
                typeof simplifiedType == 'object' &&
                simplifiedType.kind === 'tuple'
            ) {
                // Inline tuple spread: ..[A, B] → A, B
                return simplifiedType.elements;
            }
            return [{ ...element, type: simplifiedType }];
        });
        return type;
    }

    if (type.kind === 'reflection') {
        return type;
    }

    if (type.kind === 'record') {
        if ('fields' in type) {
            type.fields = type.fields.map((field) => simplifyRecordField(field, config));
        } else {
            if (type.key != null) type.key = simplifyImpl(type.key, config);
            type.value = simplifyImpl(type.value, config);
            if (config.normalizeGenericRecord && type.key === 'string') {
                delete type.key;
            }
        }
        return type;
    }

    if (type.kind === 'union') {
        let simplifiedTypes = flattenUnionTypes(
            type.types.map((item) => simplifyImpl(item, config)),
            config,
        );

        // Top-type elimination: unknown | T → unknown, any | T → any, T | never → T
        const topTypes = resolveTopTypes(config.simplifyTopTypesInUnions);
        if (topTypes.length > 0) {
            if (topTypes.includes('any') && simplifiedTypes.includes('any')) return 'any';
            if (topTypes.includes('unknown') && simplifiedTypes.includes('unknown')) return 'unknown';
            if (topTypes.includes('never')) {
                simplifiedTypes = simplifiedTypes.filter((t) => t !== 'never');
            }
        }

        if (config.deduplicateUnions) {
            simplifiedTypes = deduplicateTypeMembers(simplifiedTypes);
        }
        if (config.unwrapSingleUnion) {
            if (simplifiedTypes.length === 1) return simplifiedTypes[0]!;
            if (simplifiedTypes.length === 0) return 'never';
        }
        type.types = simplifiedTypes;
        return type;
    }

    if (type.kind === 'intersection') {
        let simplifiedTypes = flattenIntersectionTypes(
            type.types.map((item) => simplifyImpl(item, config)),
            config,
        );

        // Top-type elimination: never & T → never, T & unknown → T, T & any → T
        const topTypes = resolveTopTypes(config.simplifyTopTypesInIntersections);
        if (topTypes.length > 0) {
            const typeNames = new Set(simplifiedTypes.filter((t): t is string => typeof t === 'string'));
            if (topTypes.includes('never') && typeNames.has('never')) return 'never';
            let filtered = false;
            if (topTypes.includes('unknown') && typeNames.has('unknown')) {
                simplifiedTypes = simplifiedTypes.filter((t) => t !== 'unknown');
                filtered = true;
            }
            if (topTypes.includes('any') && typeNames.has('any')) {
                simplifiedTypes = simplifiedTypes.filter((t) => t !== 'any');
                filtered = true;
            }
            if (filtered) {
                if (simplifiedTypes.length === 1) return simplifiedTypes[0]!;
                if (simplifiedTypes.length === 0) {
                    // All members were the same eliminated top type
                    return typeNames.has('any') ? 'any' : 'unknown';
                }
            }
        }

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
