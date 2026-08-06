import type { TupleType } from '../parser.js';
import { simplifyImpl, type SimplifyImplOptions } from './impl.js';

/** Simplifies tuple types */
export function simplifyTuple(type: TupleType, options: SimplifyImplOptions): TupleType {
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
