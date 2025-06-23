import { createRequire } from 'node:module';
import type { NapiModule } from './type.js';

const require = createRequire(import.meta.url);
export const { compile, compileSync } = require('#lib') as NapiModule;
