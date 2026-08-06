import type { Type } from '../parser.js';

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
        case 'tuple':
            return `tuple:[${type.elements
                .map((e) => `${String(Boolean(e.spread))}:${getTypeDedupKey(e.type, symbols)}`)
                .join(',')}]`;
        case 'reflection':
            return `reflection:${type.name}`;
        default:
            return 'unknown';
    }
}

/** Removes duplicate members from union/intersection type member lists. */
export function deduplicateTypeMembers(types: Type[]): Type[] {
    if (types.length <= 1) return types;
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
