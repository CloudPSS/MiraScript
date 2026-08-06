import type { Type } from '../parser.js';
import { type SimplifyImplOptions, simplifyImpl } from './impl.js';
import { isTypeObject } from './utils.js';

/** Distributes intersections over unions using a cartesian product. */
export function distributeIntersectionsOverUnions(types: Type[], options: SimplifyImplOptions): Type {
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
