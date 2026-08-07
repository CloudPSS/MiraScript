import type { InputMode } from '@mirascript/mirascript';

/** Python 源代码生成请求。 */
export type PythonSourceRequest = {
    id: number;
    source: string;
    mode: InputMode;
    fileName: string;
};

/** Python 源代码生成响应。 */
export type PythonSourceResponse =
    { id: number; source: string; error?: never } | { id: number; source?: never; error: string };
