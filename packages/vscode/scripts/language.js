import fs from 'node:fs/promises';
import * as textmate from '@mirascript/textmate';

const output = new URL('../syntaxes/', import.meta.url);
await fs.mkdir(output, { recursive: true });

/**
 * Makes a list of names for the given language, including its aliases and the main name.
 * @param {string} name
 * @param {string[]?} aliases
 * @returns {string[]}
 */
function makeNameList(name, aliases) {
    const names = new Set((aliases ?? []).map((a) => a.toLowerCase()));
    names.add(name.toLowerCase());
    return Array.from(names).sort((a, b) => b.length - a.length || a.localeCompare(b));
}

for (const lang of /** @type {const} */ (['mirascript', 'mirascript-template', 'mirascript-doc'])) {
    const filename = `${lang}.tmLanguage.json`;
    const source = new URL(import.meta.resolve(`@mirascript/textmate/${filename}`));
    await fs.copyFile(source, new URL(filename, output));
    const data = textmate[lang];
    const names = makeNameList(data.name, data.aliases);
    await fs.writeFile(
        new URL(`markdown-${filename}`, output),
        JSON.stringify(
            {
                injectionSelector: 'L:text.html.markdown',
                scopeName: `markdown.${lang}.codeblock`,
                patterns: [{ include: `#${lang}-code-block` }],
                repository: {
                    [`${lang}-code-block`]: {
                        patterns: [
                            {
                                name: 'markup.fenced_code.block.markdown',
                                begin: `(^|\\G)(\\s*)(\\\`{3,}|~{3,})\\s*(?i:(${names.join('|')})(\\s+[^\`~]*)?$)`,
                                end: String.raw`(^|\G)(\2|\s{0,3})(\3)\s*$`,
                                beginCaptures: {
                                    3: { name: 'punctuation.definition.markdown' },
                                    4: { name: 'fenced_code.block.language.markdown' },
                                    5: { name: 'fenced_code.block.language.attributes.markdown' },
                                },
                                endCaptures: {
                                    3: { name: 'punctuation.definition.markdown' },
                                },
                                patterns: [
                                    {
                                        name: `meta.embedded.block.${lang}`,
                                        contentName: data.scopeName,
                                        begin: String.raw`(^|\G)(\s*)(.*)`,
                                        while: '(^|\\G)(?!\\s*([`~]{3,})\\s*$)',
                                        patterns: [{ include: data.scopeName }],
                                    },
                                ],
                            },
                        ],
                    },
                },
            },
            null,
            2,
        ),
        'utf8',
    );
}
