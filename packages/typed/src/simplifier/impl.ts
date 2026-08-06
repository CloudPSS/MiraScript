import type { Type } from '../parser.js';
import { simplifyIntersection } from './intersection.js';
import type { SimplifyOptions } from './index.js';
import { simplifyUnion } from './union.js';
import { simplifyRecord } from './record.js';
import { simplifyTuple } from './tuple.js';

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
        return simplifyTuple(type, options);
    }

    if (type.kind === 'reflection') {
        return type;
    }

    if (type.kind === 'record') {
        return simplifyRecord(type, options);
    }

    if (type.kind === 'union') {
        return simplifyUnion(type, options);
    }

    if (type.kind === 'intersection') {
        return simplifyIntersection(type, options);
    }
    /* c8 ignore next 3 */
    (type) satisfies never;
    return type;
}
