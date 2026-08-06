import type { Type } from '../parser.js';
import { deduplicateTypeMembers } from './dedup.js';
import { distributeIntersectionsOverUnions } from './distribute.js';
import type { SimplifyOptions } from './index.js';
import { mergeRecordFieldIntersections } from './merge.js';
import { flattenUnionTypes, flattenIntersectionTypes } from './flatten.js';
import { resolveTopTypes } from './top-type.js';
import { isFieldRecordType, isTypeObject } from './utils.js';
import { simplifyRecord } from './record.js';

/** Options for simplifyImpl */
export type SimplifyImplOptions = Required<SimplifyOptions>;

/** Simplifies a Type AST in place, optionally disabling individual normalization passes. */
export function simplifyImpl(type: Type, options: SimplifyImplOptions): Type {
    if (typeof type == 'symbol' || typeof type == 'string') {
        return type;
    }

    if (type.kind === 'array') {
        type.element = simplifyImpl(type.element, options);
        if (options.normalizeGenericArray && (type.element === 'any' || type.element === 'unknown')) {
            return 'array';
        }
        return type;
    }

    if (type.kind === 'function') {
        for (const param of type.params) {
            param.type = simplifyImpl(param.type, options);
        }
        if (type.returns != null) type.returns = simplifyImpl(type.returns, options);
        return type;
    }

    if (type.kind === 'literal') {
        return type;
    }

    if (type.kind === 'template') {
        type.parts = type.parts.map((part) => simplifyImpl(part, options));
        return type;
    }

    if (type.kind === 'tuple') {
        type.elements = type.elements.flatMap((element) => {
            const simplifiedType = simplifyImpl(element.type, options);
            if (
                options.expandTupleSpreads &&
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
        return simplifyRecord(type, options);
    }

    if (type.kind === 'union') {
        let simplifiedTypes = flattenUnionTypes(
            type.types.map((item) => simplifyImpl(item, options)),
            options,
        );

        // Top-type elimination: unknown | T → unknown, any | T → any, T | never → T
        const topTypes = resolveTopTypes(options.simplifyTopTypesInUnions);
        if (topTypes.length > 0) {
            if (topTypes.includes('any') && simplifiedTypes.includes('any')) return 'any';
            if (topTypes.includes('unknown') && simplifiedTypes.includes('unknown')) return 'unknown';
            if (topTypes.includes('never')) {
                simplifiedTypes = simplifiedTypes.filter((t) => t !== 'never');
            }
        }

        if (options.deduplicateUnions) {
            simplifiedTypes = deduplicateTypeMembers(simplifiedTypes);
        }
        if (options.unwrapSingleUnion) {
            if (simplifiedTypes.length === 1) return simplifiedTypes[0]!;
            if (simplifiedTypes.length === 0) return 'never';
        }
        type.types = simplifiedTypes;
        return type;
    }

    if (type.kind === 'intersection') {
        let simplifiedTypes = flattenIntersectionTypes(
            type.types.map((item) => simplifyImpl(item, options)),
            options,
        );

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

    /* c8 ignore next 3 */
    (type) satisfies never;
    return type;
}
