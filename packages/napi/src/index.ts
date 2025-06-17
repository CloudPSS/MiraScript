import { createRequire } from 'node:module';
import type { NapiModule } from './type';

const require = createRequire(import.meta.url);
export const { compile, compileSync } = require('#lib') as NapiModule;
