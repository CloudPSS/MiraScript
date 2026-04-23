/** Create an adapter factory */
export function createAdapterFactory<From extends object, To extends object>(
    create: (from: From) => To,
    setup: (from: From, to: To) => void,
    extract?: (to: To) => From,
): [(from: From) => To, (to: To) => From] {
    const symbol = Symbol('adapted');
    /** Add a unique symbol property to the adapted object */
    type ToWithSymbol = To & { [symbol]: From };
    const map = new WeakMap<From, ToWithSymbol>();
    const adapt = (from: From): To => {
        if (from == null || typeof from !== 'object') return from;

        let to = map.get(from);
        if (to == null) {
            to = create(from) as ToWithSymbol;
            if (!extract) {
                to[symbol] = from;
            }
            map.set(from, to);
        }
        setup(from, to);
        return to;
    };
    const reverse = (to: To): From => {
        if (to == null || typeof to !== 'object') return to;

        if (extract) {
            const result = extract(to);
            if (result != null) return result;
            throw new Error('The object is not adapted');
        }

        const from = (to as ToWithSymbol)[symbol];
        if (from == null) {
            throw new Error('The object is not adapted');
        }
        return from;
    };
    return [adapt, reverse];
}
