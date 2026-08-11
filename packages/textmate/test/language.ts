import { test } from 'node:test';
import assert from 'node:assert/strict';
import { grammars } from '../src/index.ts';
import { mirascriptLanguage } from '../src/language.ts';

test('uses the shared MiraScript language metadata', () => {
    assert.equal(grammars[0].name, mirascriptLanguage.name);
    assert.equal(grammars[0].scopeName, mirascriptLanguage.scopeName);
    assert.deepEqual(grammars[0].aliases, mirascriptLanguage.aliases);
});
