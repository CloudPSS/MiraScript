import type { IRange } from '../../monaco-api.js';
import { DiagnosticCode, type SourceDiagnostic, type SourceReference } from '@mirascript/mirascript/subtle';

/** 源代码定义信息 */
interface SourceDefinitionBase<R extends DiagnosticCode = DiagnosticCode> {
    /** 符号引用 */
    readonly references: ReadonlyArray<SourceDiagnostic<R> | SourceReference<R>>;
}

/** 局部变量类型 */
export type LocalVariableType = (typeof LocalVariableType)[number];
export const LocalVariableType = [
    DiagnosticCode.LocalMutable,
    DiagnosticCode.LocalImmutable,
    DiagnosticCode.LocalConst,
    DiagnosticCode.LocalFunction,
    DiagnosticCode.LocalModule,
] as const;
/** 显式参数类型 */
export type ParameterExplicitType = (typeof ParameterExplicitType)[number];
export const ParameterExplicitType = [
    DiagnosticCode.ParameterMutable,
    DiagnosticCode.ParameterImmutable,
    DiagnosticCode.ParameterMutableRest,
    DiagnosticCode.ParameterImmutableRest,
] as const;
/** 子模式参数类型 */
export type ParameterSubPatternType = (typeof ParameterSubPatternType)[number];
export const ParameterSubPatternType = [
    DiagnosticCode.ParameterSubPatternImmutable,
    DiagnosticCode.ParameterSubPatternMutable,
] as const;
/** 模式参数类型 */
export type ParameterPatternType = (typeof ParameterPatternType)[number];
export const ParameterPatternType = [DiagnosticCode.ParameterPattern, DiagnosticCode.ParameterRestPattern] as const;
/** 隐式参数类型 */
export type ParameterItType = (typeof ParameterItType)[number];
export const ParameterItType = [DiagnosticCode.ParameterIt] as const;
/** 参数定义类型 */
export type ParameterDefinitionType = (typeof ParameterDefinitionType)[number];
export const ParameterDefinitionType = [
    ...ParameterExplicitType,
    ...ParameterSubPatternType,
    ...ParameterItType,
] as const;
/** 参数占位符类型 */
export type ParameterPlaceholderType = (typeof ParameterPlaceholderType)[number];
export const ParameterPlaceholderType = [
    ...ParameterExplicitType,
    ...ParameterPatternType,
    ...ParameterItType,
] as const;
/** 局部定义类型 */
export type LocalDefinitionType = (typeof LocalDefinitionType)[number];
export const LocalDefinitionType = [...LocalVariableType, ...ParameterDefinitionType] as const;

/** 源代码定义信息 */
export interface LocalDefinition<T extends LocalDefinitionType = LocalDefinitionType> extends SourceDefinitionBase<
    | DiagnosticCode.ReadLocal
    | DiagnosticCode.WriteLocal
    | DiagnosticCode.ReadWriteLocal
    | DiagnosticCode.RedeclareLocal
    | DiagnosticCode.ExportedLocal
> {
    /** 符号定义 */
    readonly definition: SourceDiagnostic<T>;
    /** 定义的函数，仅对 LocalFunction 有效 */
    readonly fn?: {
        /** 函数作用域 */
        scope: SourceScope;
        /** 函数的参数 */
        args: ReadonlyArray<LocalDefinition<ParameterDefinitionType>>;
    };
}
/** 源代码定义信息 */
export interface GlobalDefinition extends SourceDefinitionBase<DiagnosticCode.GlobalVariable> {
    /** 符号名称 */
    readonly name: string;
}
/** 源代码定义信息 */
export type SourceDefinition = LocalDefinition | GlobalDefinition;

/** 作用域信息 */
export interface SourceScope {
    /** 作用域范围 */
    readonly range: IRange;
    /** 包含的局部变量 */
    readonly locals: readonly LocalDefinition[];
    /** 包含的参数占位符 */
    readonly params: ReadonlyArray<SourceDiagnostic<ParameterPlaceholderType>>;
    /** 包含的作用域 */
    readonly children: readonly SourceScope[];
    /** 父作用域 */
    readonly parent?: SourceScope;
}
