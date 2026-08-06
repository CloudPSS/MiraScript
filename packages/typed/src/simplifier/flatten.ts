import type { Type } from '../parser.js';
import type { SimplifyImplOptions } from './impl.js';
import { isTypeObject } from './utils.js';

/** Flattens nested union nodes when the corresponding option is enabled. */
export function flattenUnionTypes(types: Type[], options: SimplifyImplOptions): Type[] {
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
export function flattenIntersectionTypes(types: Type[], options: SimplifyImplOptions): Type[] {
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
