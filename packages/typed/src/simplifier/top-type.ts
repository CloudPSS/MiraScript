/** Top types that can be absorbed / eliminated in simplification. */
export type TopType = 'unknown' | 'never' | 'any';

/** Resolves the top types option into an array of top types. */
export function resolveTopTypes(value: boolean | TopType[] | undefined): TopType[] {
    if (value === false || value == null) return [];
    if (value === true) return ['unknown', 'never', 'any'];
    return value;
}
