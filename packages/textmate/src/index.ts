import type { LanguageRegistration } from '@shikijs/types';
import { createMiraScriptDocGrammar, createMiraScriptGrammar, createMiraScriptTemplateGrammar } from './grammar.ts';

export const mirascript = createMiraScriptGrammar();
export const mirascriptTemplate = createMiraScriptTemplateGrammar();
export const mirascriptDoc = createMiraScriptDocGrammar();

export { mirascriptTemplate as 'mirascript-template', mirascriptDoc as 'mirascript-doc' };

export const grammars: LanguageRegistration[] = [mirascript, mirascriptTemplate, mirascriptDoc];
