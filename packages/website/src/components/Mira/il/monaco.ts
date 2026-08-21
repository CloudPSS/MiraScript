const MIRASCRIPT_IL = 'mirascript-il';

/** 注册 MiraScript IL 的基础 Monarch 高亮。 */
export function registerIL(monaco: typeof import('@private/monaco-editor')): void {
    monaco.languages.register({ id: MIRASCRIPT_IL });
    monaco.languages.setMonarchTokensProvider(MIRASCRIPT_IL, {
        defaultToken: '',
        tokenizer: {
            root: [
                [/;.*$/, 'comment'],
                [/^\s*\.\w+\s*$/, 'keyword.directive'],
                [/^[0-9a-f]{8}\b/, 'number.hex'],
                [/\b[A-Z][A-Z_]*(?:\.WIDE)?\b/, 'keyword'],
                [/%\d+\b/, 'variable'],
                [/#\d+\b/, 'constant'],
                [/'/, { token: 'string', next: '@string' }],
                [/\b(?:true|false|nil)\b/, 'constant.language'],
                [/\b[+-]?(?:nan|inf)\b/, 'number.constant'],
                [/[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?\b/, 'number'],
                [/[,.=()]/, 'delimiter'],
            ],
            string: [
                [/'/, { token: 'string', next: '@pop' }],
                [/[^'\\]+/, 'string'],
                [/\\x[0-9A-Fa-f]{2}/, 'string.escape'],
                [/\\u\{[0-9A-Fa-f]+\}/, 'string.escape'],
                [/\\./, 'string.escape'],
            ],
        },
    });
}
