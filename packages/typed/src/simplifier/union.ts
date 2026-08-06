import type { Type, UnionType } from '../parser.js';
import { deduplicateTypeMembers } from './dedup.js';
import { simplifyImpl, type SimplifyImplOptions } from './impl.js';
import { resolveTopTypes } from './top-type.js';
import { isTypeObject } from './utils.js';

/** Flattens nested union nodes when the corresponding option is enabled. */
function flattenUnionTypes(type: UnionType, options: SimplifyImplOptions): Type[] {
    const types = type.types.map((item) => simplifyImpl(item, options));
    if (!options.flattenUnions) return types;
    const result: Type[] = [];
    for (const type of types) {
        if (isTypeObject(type) && type.kind === 'union') {
            result.push(...flattenUnionTypes(type, options));
        } else {
            result.push(type);
        }
    }
    return result;
}

/** Simplifies a union recursively. */
export function simplifyUnion(type: UnionType, options: SimplifyImplOptions): Type {
    let simplifiedTypes = flattenUnionTypes(type, options);

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
