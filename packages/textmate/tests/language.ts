import test from 'ava';
import { grammars } from '../src/index.ts';
import { mirascriptLanguage } from '../src/language.ts';

test('uses the shared MiraScript language metadata', (t) => {
    t.is(grammars[0].name, mirascriptLanguage.name);
    t.is(grammars[0].scopeName, mirascriptLanguage.scopeName);
    t.deepEqual(grammars[0].aliases, mirascriptLanguage.aliases);
});
