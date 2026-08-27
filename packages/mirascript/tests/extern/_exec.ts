import { compileSync, type VmContext, type VmValue } from '@mirascript/mirascript';

export const exec = (context: VmContext): ((source: string) => VmValue) => {
    return (source: string) => {
        const script = compileSync(source);
        return script(context);
    };
};
