const REMAP_PREFIXES: ReadonlyArray<[string, string]> = [
    ['constant.character.escape.', 'string.escape.'],
    ['constant.numeric.', 'number.'],
    ['constant.language.', 'keyword.'],
    ['support.variable.', 'variable.'],
    ['entity.name.type.', 'type.'],
    ['support.type.', 'type.'],
    ['entity.name.namespace.', 'namespace.'],
    ['entity.name.function.', 'function.'],
];

/** Remap scope */
function remapScope(scope: string): string {
    for (const [prefix, remap] of REMAP_PREFIXES) {
        if (scope.startsWith(prefix)) {
            return remap + scope.slice(prefix.length);
        }
    }
    return scope;
}

const STYLED_SCOPE_PREFIXES = [
    'invalid.',
    'comment.',
    'string.',
    'keyword.',
    'constant.',
    'variable.',
    'entity.',
    'storage.',
    'support.',
    'markup.',
    'meta.embedded.',
];
/** Is styled scope */
function isStyledScope(scope: string): boolean {
    return STYLED_SCOPE_PREFIXES.some((prefix) => scope.startsWith(prefix));
}

/** Select the deepest scope that a native Monaco theme can style. */
export function textmateScopesToMonaco(scopes: readonly string[]): string {
    let fallback;
    for (let index = scopes.length - 1; index >= 0; index -= 1) {
        const scope = scopes[index]!;
        fallback ||= scope;
        if (isStyledScope(scope)) return remapScope(scope);
    }
    return fallback ? remapScope(fallback) : 'source.mira';
}
