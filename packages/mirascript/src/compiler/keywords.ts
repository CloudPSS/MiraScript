import './load-module.js';

import { getModule } from '@mirascript/bindings';
/** 所有 MiraScript 关键字 */
export let keywords: () => readonly string[] = () => {
    const kw = Object.freeze(getModule().keywords());
    keywords = () => kw;
    return kw;
};
