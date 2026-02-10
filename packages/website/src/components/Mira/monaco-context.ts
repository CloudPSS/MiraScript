import { createVmContext, type VmAny, type VmContext } from '@mirascript/mirascript';
import type { editor } from '@private/monaco-editor';

const contextMap = new WeakMap<editor.ITextModel, Record<string, VmAny>>();

/** 设置 context */
export function setMonacoContext(model: editor.ITextModel | null, context: Record<string, VmAny> | undefined): void {
    if (model == null || context == null) return;
    contextMap.set(model, context);
}

/** 获取 context */
export function getMonacoContext(model: editor.ITextModel): VmContext {
    return createVmContext(contextMap.get(model));
}
