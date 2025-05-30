/**
 * 将只读属性转换为可写属性
 */
export type Mutable<T> = { -readonly [K in keyof T]: T[K] };
