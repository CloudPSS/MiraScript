import { createHighlighter } from 'shiki';
import fs from 'node:fs/promises';
import { grammars } from '../src/index.ts';

const dist = new URL('../dist/', import.meta.url);
await fs.mkdir(dist, { recursive: true });

const highlighter = await createHighlighter({
    langs: grammars,
    themes: [],
});

try {
    for (const grammar of grammars) {
        highlighter.getLanguage(grammar.name).tokenizeLine('let value = "$(1 + 2)";', null);
    }
} finally {
    highlighter.dispose();
}

for (const grammar of grammars) {
    const filename = `${grammar.name}.tmLanguage.json`;
    await fs.writeFile(
        new URL(filename, dist),
        `${JSON.stringify(
            {
                $schema: 'https://www.schemastore.org/tmlanguage.json',
                ...grammar,
            },
            null,
            2,
        )}\n`,
        'utf8',
    );
}
