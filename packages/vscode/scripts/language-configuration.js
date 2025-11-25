import { configuration } from '@mirascript/monaco/basic';
import fs from 'node:fs/promises';

await fs.writeFile(
    new URL('../language-configuration.json', import.meta.url),
    JSON.stringify(
        configuration(),
        (r, v) => {
            if (v instanceof RegExp) {
                return v.source;
            }
            return v;
        },
        2,
    ),
    'utf8',
);
