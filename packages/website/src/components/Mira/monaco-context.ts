import { createVmContext, isVmContext, type VmAny, type VmContext } from '@mirascript/mirascript';
import type { editor } from '@private/monaco-editor';

const contextMap = new WeakMap<editor.ITextModel, VmContext>();

/** 设置 context */
export function setMonacoContext(
    model: editor.ITextModel | null,
    context: Record<string, VmAny> | VmContext | undefined,
): void {
    if (model == null || context == null) return;
    if (!isVmContext(context)) {
        context = createVmContext(context);
    }
    contextMap.set(model, context);
}

/** 获取 context */
export function getMonacoContext(model: editor.ITextModel): VmContext {
    return contextMap.get(model) ?? createVmContext();
}
