/** MiraScript 执行上下文 */
type VmContext = {
    /**
     * 获取指定 key 的值 `global[key]`
     * @throws {VmError} 如果值不存在则抛出异常
     */
    get(key: string): VmValue;
    /** 查找指定 key 是否存在 `key in global` */
    has(key: string): boolean;
};
/** Mirascript 虚拟机内的未初始化变量 */
type VmUninitialized = undefined;
/** Mirascript 原始值 */
type VmPrimitive = null | number | string | boolean;
/**
 * Mirascript 数组
 * 数组中的 `undefined`、`null` 及 <empty slot> 均视作 `nil`
 */
type VmArray = ReadonlyArray<VmConst | undefined>;
/**
 * Mirascript 记录
 * 仅拥有且可枚举的字符串键视作存在
 * 字段值 `undefined` 和 `null` 均视作 `nil`
 */
type VmRecord = {
    readonly [key: string]: VmConst | undefined;
};
/** Mirascript 虚拟机内的值语义值 */
type VmConst = VmPrimitive | VmRecord | VmArray;
/** Mirascript 虚拟机内的不可变值 */
type VmImmutable = VmConst | VmFunction | VmModule;
/** Mirascript 虚拟机内的合法值 */
type VmValue = VmImmutable | VmExtern;
/** Mirascript 虚拟机内的值（包括未初始化变量） */
type VmAny = VmValue | VmUninitialized;
/** Mirascript 模块 */
class VmModule<const T extends Record<string, VmImmutable> = Record<string, VmImmutable>> {
    /** 模块名称 */
    readonly name: string;
    /** 模块导出 */
    readonly value: T;
}
/** 包装 Mirascript `extern` 类型的对象 */
class VmExtern<const T extends object = object> {
    /** 包装值 */
    readonly value: T;
    /** 当 {@link value} 是函数时，绑定的 this 参数 */
    readonly thisArg: ThisParameterType<T> | null = null;
}
/**
 * Mirascript 函数签名
 *
 * 虽然所有输入参数的类型均为 {@linkcode VmValue}，但当参数不足时，对应的参数会被填充为 `undefined`。
 */
type VmFunctionLike = (...args: ReadonlyArray<VmValue | undefined>) => VmAny;
/** Mirascript 函数 */
type VmFunction<T extends VmFunctionLike = VmFunctionLike> = T;

/** 检查点 */
function $CpEnter(): void;
/** 检查点 */
function $CpExit(): void;
/** 检查点 */
function $Cp(): void;
/** 调用返回类型 */
type CallReturn<T extends VmValue> =
    T extends VmFunction<infer F>
        ? F extends (...args: readonly VmAny[]) => infer R
            ? // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
              R extends void | undefined
                ? null
                : R
            : VmValue
        : T extends VmExtern<infer E>
          ? E extends (...args: readonly VmAny[]) => infer R
              ? // eslint-disable-next-line @typescript-eslint/no-invalid-void-type
                R extends void | undefined
                  ? null
                  : R extends VmValue
                    ? R
                    : R extends object
                      ? VmExtern<R>
                      : VmValue
              : VmValue
          : VmValue;
/** 调用函数 */
function $Call<T extends VmValue, A extends readonly VmValue[]>(func: T, args: A): CallReturn<T>;
/** 过滤剩余参数数组 */
function $VArgs(varags: VmAny[]): VmArray;
/** 断言值已初始化 */
function $AssertInit(value: VmAny): asserts value is VmValue;
/** 检查值是否在可迭代对象中 */
function $In(value: VmAny, iterable: VmAny): boolean;
/** 获取值的长度 */
function $Length(value: VmAny): number;
/** 删除记录中的指定字段 */
function $Omit(value: VmAny, omitted: ReadonlyArray<number | string>): VmRecord;
/** 选择记录中的指定字段 */
function $Pick(value: VmAny, picked: ReadonlyArray<number | string>): VmRecord;
/** 检查是否拥有字段 */
function $Has(obj: VmAny, key: VmAny): boolean;
/** 获取字段 */
function $Get(obj: VmAny, key: VmAny): VmValue;
/** 设置字段 */
function $Set(obj: VmAny, key: VmAny, value: VmAny): void;
/** 获取可迭代对象 */
function $Iterable(value: VmAny): Iterable<VmValue | undefined>;
/** 展开记录 */
function $RecordSpread(record: VmAny): VmRecord | null;
/** 展开数组 */
function $ArraySpread(array: VmAny): Iterable<VmConst | undefined>;
/** 转换为布尔值 */
function $ToBoolean(value: VmAny): boolean;
/** 转换为字符串 */
function $ToString(value: VmAny): string;
/** 转换为数字 */
function $ToNumber(value: VmAny): number;
/** 格式化值 */
function $Format(value: VmAny, format: string | null): string;
/** 构造 module */
function $Module<const T extends Record<string, () => VmImmutable>>(
    name: string,
    body: T,
): VmModule<{ [K in keyof T]: ReturnType<T[K]> }>;
/** 构造 record | array 元素 */
function $El(value: VmAny): VmConst;
/** 构造 record 可选元素 */
function $ElOpt(key: string, value: VmAny): VmConst;
/** 构造函数和函数表达式 */
function $Fn<T extends VmFunctionLike>(name: string | null | undefined, fn: T): VmFunction<T>;
/** 读取闭包上值 */
function $Upvalue(value: VmAny): VmValue;
/** 构造范围数组 */
function $ArrayRange(start: VmAny, end: VmAny): VmArray;
/** 构造范围数组（不包含结束值） */
function $ArrayRangeExclusive(start: VmAny, end: VmAny): VmArray;
/** 默认执行上下文 */
function $GlobalFallback(): VmContext;
/** 字符串连接 */
function $Concat(...args: readonly string[]): string;
/** 正号 */
function $Pos(a: VmAny): number;
/** 负号 */
function $Neg(a: VmAny): number;
/** 非 */
function $Not(a: VmAny): boolean;
/** 加法 */
function $Add(a: VmAny, b: VmAny): number;
/** 减法 */
function $Sub(a: VmAny, b: VmAny): number;
/** 乘法 */
function $Mul(a: VmAny, b: VmAny): number;
/** 除法 */
function $Div(a: VmAny, b: VmAny): number;
/** 取模 */
function $Mod(a: VmAny, b: VmAny): number;
/** 乘方 */
function $Pow(a: VmAny, b: VmAny): number;
/** 与 */
function $And(a: VmAny, b: VmAny): boolean;
/** 或 */
function $Or(a: VmAny, b: VmAny): boolean;
/** 大于 */
function $Gt(a: VmAny, b: VmAny): boolean;
/** 大于等于 */
function $Gte(a: VmAny, b: VmAny): boolean;
/** 小于 */
function $Lt(a: VmAny, b: VmAny): boolean;
/** 小于等于 */
function $Lte(a: VmAny, b: VmAny): boolean;
/** 等于 */
function $Eq(a: VmAny, b: VmAny): boolean;
/** 不等于 */
function $Neq(a: VmAny, b: VmAny): boolean;
/** 近似等于 */
function $Aeq(a: VmAny, b: VmAny): boolean;
/** 不近似等于 */
function $Naeq(a: VmAny, b: VmAny): boolean;
/** 全等于 */
function $Same(a: VmAny, b: VmAny): boolean;
/** 不全等于 */
function $Nsame(a: VmAny, b: VmAny): boolean;
/** 获取数组切片 */
function $Slice(value: VmAny, start: VmAny, end: VmAny): VmArray;
/** 获取数组切片（不包含结束位置） */
function $SliceExclusive(value: VmAny, start: VmAny, end: VmAny): VmArray;
/** 获取值的类型名称 */
function $Type(value: VmAny): TypeName;
/** 判断值是否为布尔值 */
function $IsBoolean(value: VmAny): value is boolean;
/** 判断值是否为数字 */
function $IsNumber(value: VmAny): value is number;
/** 判断值是否为字符串 */
function $IsString(value: VmAny): value is string;
/** 判断值是否为记录 */
function $IsRecord(value: VmAny): value is VmRecord;
/** 判断值是否为数组 */
function $IsArray(value: VmAny): value is VmArray;
/** 断言值非 nil */
function $AssertNonNil(value: VmAny): asserts value is NonNullable<VmValue>;
