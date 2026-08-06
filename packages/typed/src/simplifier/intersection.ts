import type { IntersectionType, Type, RecordType, RecordField } from '../parser.js';
import { deduplicateTypeMembers } from './dedup.js';
import { type SimplifyImplOptions, simplifyImpl } from './impl.js';
import { resolveTopTypes } from './top-type.js';
import { isFieldRecordType, isTypeObject } from './utils.js';

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

/** Flattens nested intersection nodes when the corresponding option is enabled. */
function flattenIntersectionTypes(type: IntersectionType, options: SimplifyImplOptions): Type[] {
    const types = type.types.map((item) => simplifyImpl(item, options));
    if (!options.flattenIntersections) return types;
    const result: Type[] = [];
    for (const type of types) {
        if (isTypeObject(type) && type.kind === 'intersection') {
            result.push(...flattenIntersectionTypes(type, options));
        } else {
            result.push(type);
        }
    }
    return result;
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

    const opt = { ...options, distributeIntersectionsOverUnions: false };
    const branches = combinations.map((combination) => simplifyImpl({ kind: 'intersection', types: combination }, opt));
    if (branches.length === 1) return branches[0]!;
    return { kind: 'union', types: branches };
}

/** Simplifies an intersection type */
export function simplifyIntersection(type: IntersectionType, options: SimplifyImplOptions): Type {
    let simplifiedTypes = flattenIntersectionTypes(type, options);

    // Top-type elimination: never & T → never, T & unknown → T, T & any → T
    const topTypes = resolveTopTypes(options.simplifyTopTypesInIntersections);
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

    if (options.deduplicateIntersections) {
        simplifiedTypes = deduplicateTypeMembers(simplifiedTypes);
    }
    if (
        options.distributeIntersectionsOverUnions &&
        simplifiedTypes.some((item) => isTypeObject(item) && item.kind === 'union')
    ) {
        return distributeIntersectionsOverUnions(simplifiedTypes, options);
    }
    if (options.mergeRecordIntersections) {
        const recordTypes = simplifiedTypes.filter(isFieldRecordType);
        if (recordTypes.length >= 2) {
            const nonRecordTypes = simplifiedTypes.filter((item) => !isFieldRecordType(item));
            const mergedRecord = simplifyImpl(mergeRecordFieldIntersections(recordTypes), options);
            let mergedTypes = [mergedRecord, ...nonRecordTypes];
            if (options.deduplicateIntersections) {
                mergedTypes = deduplicateTypeMembers(mergedTypes);
            }
            if (options.unwrapSingleIntersection && mergedTypes.length === 1) {
                return mergedTypes[0]!;
            }
            type.types = mergedTypes;
            return type;
        }
    }
    if (options.unwrapSingleIntersection && simplifiedTypes.length === 1) {
        return simplifiedTypes[0]!;
    }
    type.types = simplifiedTypes;
    return type;
}
