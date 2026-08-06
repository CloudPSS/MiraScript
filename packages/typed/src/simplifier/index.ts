import type { Type } from '../parser.js';
import { simplifyImpl } from './impl.js';
import type { TopType } from './top-type.js';

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
    /** Inline tuple spread elements (..[A, B] → A, B). */
    expandTupleSpreads?: boolean;
    /**
     * Eliminate / absorb top types in unions.
     * - `unknown | T` → `unknown`
     * - `any | T` → `any`
     * - `T | never` → `T`
     * `true` enables all, or pass a subset of `'unknown' | 'never' | 'any'`.
     */
    simplifyTopTypesInUnions?: boolean | TopType[];
    /**
     * Eliminate / absorb top types in intersections.
     * - `never & T` → `never`
     * - `T & unknown` → `T`
     * - `T & any` → `T`
     * `true` enables all, or pass a subset of `'unknown' | 'never' | 'any'`.
     */
    simplifyTopTypesInIntersections?: boolean | TopType[];
    /** `record<string, V>` → `record<V>` (string is the default key). */
    normalizeGenericRecord?: boolean;
    /** `array<any | unknown>` → `array` (no element constraint). */
    normalizeGenericArray?: boolean;
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
    expandTupleSpreads: true,
    simplifyTopTypesInUnions: true,
    simplifyTopTypesInIntersections: true,
    normalizeGenericRecord: true,
    normalizeGenericArray: true,
};

/** Simplifies a Type AST in place, optionally disabling individual normalization passes. */
export function simplify(type: Type, options?: SimplifyOptions): Type {
    const config = { ...DEFAULT_OPTIONS, ...options };
    return simplifyImpl(type, config);
}
