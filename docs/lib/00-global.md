# 全局

### `E`: `number`

自然对数的底数

### `e`: `number`

:::warning[已弃用]
常量 E 建议使用大写
:::

自然对数的底数

### `LN10`: `number`

10 的自然对数

### `LN2`: `number`

2 的自然对数

### `LOG10E`: `number`

e 以 10 为底的对数

### `LOG2E`: `number`

e 以 2 为底的对数

### `PI`: `number`

圆周率

### `pi`: `number`

:::warning[已弃用]
常量 PI 建议使用大写
:::

圆周率

### `SQRT1_2`: `number`

½ 的平方根

### `SQRT2`: `number`

2 的平方根

### `fn abs(x)`

返回数值的绝对值

**参数**

- `x`: `number`: 要取绝对值的数

**返回值** `number`

### `fn acos(x)`

返回数值的反余弦值（弧度）

**参数**

- `x`: `number`: 要计算反余弦的数

**返回值** `number`

### `fn acosh(x)`

返回数值的反双曲余弦值

**参数**

- `x`: `number`: 要计算反双曲余弦的数

**返回值** `number`

### `fn all(data, predicate)`

检查数组或记录中的所有键值对是否都满足条件

**参数**

- `data`: `array | record`: 要检查的数组或记录
- `predicate`: `fn(value: any, key: number | string, input: type(data)) -> boolean`: 用于测试每个键值对的函数，返回 true 或 false

**返回值** `boolean`

**示例**

```mira
all([1, 2, 3], fn { it > 0 }) // true
```

### `fn any(data, predicate)`

检查数组或记录中的是否存在满足条件的键值对

**参数**

- `data`: `array | record`: 要检查的数组或记录
- `predicate`: `fn(value: any, key: number | string, input: type(data)) -> boolean`: 用于测试每个键值对的函数，返回 true 或 false

**返回值** `boolean`

**示例**

```mira
any([0, 1, 2], fn { it > 1 }) // true
```

### `fn asin(x)`

返回数值的反正弦值（弧度）

**参数**

- `x`: `number`: 要计算反正弦的数

**返回值** `number`

### `fn asinh(x)`

返回数值的反双曲正弦值

**参数**

- `x`: `number`: 要计算反双曲正弦的数

**返回值** `number`

### `fn atan(x)`

返回数值的反正切值（弧度）

**参数**

- `x`: `number`: 要计算反正切的数

**返回值** `number`

### `fn atan2(x, y)`

返回从原点到点 (x, y) 的角度（弧度）

**参数**

- `x`: `number`: x 坐标
- `y`: `number`: y 坐标

**返回值** `number`

### `fn atanh(x)`

返回数值的反双曲正切值

**参数**

- `x`: `number`: 要计算反双曲正切的数

**返回值** `number`

### `fn b_and(x, y)`

返回两个数的按位与

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 第二个操作数

**返回值** `number`

**示例**

```mira
b_and(6, 3) // 2
```

### `fn b_not(x)`

返回一个数的按位取反

**参数**

- `x`: `number`: 操作数

**返回值** `number`

**示例**

```mira
b_not(0) // -1
```

### `fn b_or(x, y)`

返回两个数的按位或

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 第二个操作数

**返回值** `number`

**示例**

```mira
b_or(5, 2) // 7
```

### `fn b_xor(x, y)`

返回两个数的按位异或

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 第二个操作数

**返回值** `number`

**示例**

```mira
b_xor(5, 3) // 6
```

### `fn cbrt(x)`

返回数值的立方根

**参数**

- `x`: `number`: 要计算立方根的数

**返回值** `number`

### `fn ceil(x)`

返回大于等于给定数的最小整数

**参数**

- `x`: `number`: 要向上取整的数

**返回值** `number`

### `fn chars(str)`

将字符串转换为字符数组

**参数**

- `str`: `string`: 要转换的字符串

**返回值** `string[]`

**示例**

```mira
chars("Mira") // ["M", "i", "r", "a"]
```

### `fn contains(str, search)`

检查字符串是否包含指定子串

**参数**

- `str`: `string`: 要检查的字符串
- `search`: `string`: 要匹配的子串

**返回值** `boolean`

**示例**

```mira
contains("hello", "ll") // true
```

### `fn cos(x)`

返回数值的余弦值

**参数**

- `x`: `number`: 要计算余弦的数（弧度）

**返回值** `number`

### `fn cosh(x)`

返回数值的双曲余弦值

**参数**

- `x`: `number`: 要计算双曲余弦的数

**返回值** `number`

### `fn debug_print(..args)`

打印调试信息到控制台

**参数**

- `..args`: `any[]`: 要打印的调试信息，可以是任意类型

**返回值** `nil`

**示例**

```mira
debug_print("value:", 42);
```

### `fn ends_with(str, search)`

检查字符串是否以指定子串结尾

**参数**

- `str`: `string`: 要检查的字符串
- `search`: `string`: 要匹配的子串

**返回值** `boolean`

**示例**

```mira
ends_with("mira", "ra") // true
```

### `fn entries(data)`

返回数组或记录的键值对列表

**参数**

- `data`: `array | record`: 要获取键值对的数组或记录

**返回值** `(number, any)[] | (string, any)[]`

**示例**

```mira
entries([1]) // [(0, 1)]
entries((a: 1)) // [("a", 1)]
```

### `fn exp(x)`

返回 e 的指定次幂

**参数**

- `x`: `number`: 指数

**返回值** `number`

### `fn expm1(x)`

返回 e 的 x 次幂减 1

**参数**

- `x`: `number`: 指数

**返回值** `number`

### `fn factorial(x)`

返回一个数的阶乘

**参数**

- `x`: `number`: 要计算阶乘的数值

**返回值** `number`

**示例**

```mira
factorial(5) // 120
```

### `fn filter(data, predicate)`

过滤数组或记录中的元素，返回满足条件的元素

**参数**

- `data`: `array | record`: 要过滤的数组或记录
- `predicate`: `fn(value: any, key: number | string, input: type(data)) -> boolean`: 用于测试每个元素的函数

**返回值** `type(data)`

**示例**

```mira
filter([1, 2, 3, 4], fn (v) { v % 2 == 0 }) // [2, 4]
```

### `fn filter_map(data, f)`

对数组或记录中的每个元素应用函数，并返回非 nil 的结果

**参数**

- `data`: `array | record`: 要映射的数组或记录
- `f`: `fn(value: any, key: number | string, input: type(data)) -> any | nil`: 应用于每个元素的函数

**返回值** `type(data)`

**示例**

```mira
filter_map([1, 2, 3], fn (v) { if v % 2 == 0 { v * v } else { nil } }) // [4]
```

### `fn find(data, predicate)`

查找数组或记录中的键值对，返回第一个满足条件的键值对

**参数**

- `data`: `array | record`: 要查找的数组或记录
- `predicate`: `(fn(value: any, key: number | string, input: type(data)) -> boolean) | any`: 用于测试每个键值对的函数，或要查找的值

**返回值** `(number | string, any) | nil`

**示例**

```mira
find([3, 5, 8], fn (v) { v % 2 == 0 }) // (2, 8)
find((x: 1, y: 2, z: 3), 2) // ('y', 2)
```

### `fn flatten(data, depth)`

将数组扁平化

**参数**

- `data`: `array`: 要扁平化的数组
- `depth`: `number`: 扁平化的深度，默认为 1

**返回值** `array`

**示例**

```mira
flatten([[1, 2], [3, [4]]], 2) // [1, 2, 3, 4]
```

### `fn floor(x)`

返回小于等于给定数的最大整数

**参数**

- `x`: `number`: 要向下取整的数

**返回值** `number`

### `fn format(data, format)`

将数据格式化为指定格式的字符串

**参数**

- `data`: `any`: 要格式化的数据
- `format`: `string`: 格式字符串

**返回值** `string`

**示例**

```mira
format(12, ".3") // "12.000"
```

### `fn from_json(json, fallback)`

将 JSON 字符串转换为数据

**参数**

- `json`: `string`: 要转换的 JSON 字符串
- `fallback`: `any`: 如果转换失败，返回的默认值

**返回值** `any`

**示例**

```mira
from_json('{"a":1}') // (a: 1)
```

### `fn gamma(x)`

返回 Gamma 函数的值

**参数**

- `x`: `number`: 要计算 Gamma 函数的数值

**返回值** `number`

**示例**

```mira
gamma(5) // 24
```

### `fn hypot(..values)`

返回所有参数平方和的平方根

**参数**

- `..values`: `number[]`: 要计算的数值

**返回值** `number`

**示例**

```mira
hypot(3, 4) // 5
```

### `fn join(arr, separator)`

将字符串数组连接为单个字符串

**参数**

- `arr`: `string[]`: 要连接的字符串数组
- `separator`: `string`: 分隔符

**返回值** `string`

**示例**

```mira
join(["a", "b", "c"], "-") // "a-b-c"
```

### `fn keys(data)`

返回数组、记录、外部对象或模块的键列表

**参数**

- `data`: `array | record | extern | module`: 要获取键的数组、记录、外部对象或模块

**返回值** `number[] | string[]`

**示例**

```mira
keys([10, 20]) // [0, 1]
keys((10, 20)) // ["0", "1"]
```

### `fn len(arr)`

返回数组的长度

**参数**

- `arr`: `array | extern`: 要求长度的数组

**返回值** `number`

**示例**

```mira
len([1, 2, 3]) // 3
```

### `fn log(x)`

返回数值的自然对数（以 e 为底）

**参数**

- `x`: `number`: 要取对数的数

**返回值** `number`

### `fn log10(x)`

返回数值的以 10 为底的对数

**参数**

- `x`: `number`: 要取对数的数

**返回值** `number`

### `fn log1p(x)`

返回 1 加上数值的自然对数

**参数**

- `x`: `number`: 要取对数的数

**返回值** `number`

### `fn log2(x)`

返回数值的以 2 为底的对数

**参数**

- `x`: `number`: 要取对数的数

**返回值** `number`

### `fn map(data, f)`

对数组或记录中的每个元素应用函数，并返回结果

**参数**

- `data`: `array | record`: 要映射的数组或记录
- `f`: `fn(value: any, key: number | string, input: type(data)) -> any`: 应用于每个元素的函数

**返回值** `type(data)`

**示例**

```mira
map([1, 2, 3], fn (v) { v * v }) // [1, 4, 9]
```

### `fn max(..values)`

返回一组数中的最大值

**参数**

- `..values`: `number[]`: 要比较的数值

**返回值** `number`

**示例**

```mira
max(3, 7, 2) // 7
```

### `fn min(..values)`

返回一组数中的最小值

**参数**

- `..values`: `number[]`: 要比较的数值

**返回值** `number`

**示例**

```mira
min(3, 7, 2) // 2
```

### `fn panic(message)`

产生错误，并打印错误信息到控制台

**参数**

- `message`: `string`: 要打印的错误信息

**返回值** `never`

**示例**

```mira
panic("boom");
```

### `fn pow(x, y)`

返回 x 的 y 次幂

**参数**

- `x`: `number`: 底数
- `y`: `number`: 指数

**返回值** `number`

### `fn product(..values)`

返回一组数的乘积

**参数**

- `..values`: `number[]`: 要计算的数值

**返回值** `number`

**示例**

```mira
product(2, 3, 4) // 24
```

### `fn random()`

返回 [0, 1) 之间的伪随机数

**返回值** `number`

### `fn repeat(data, times)`

创建一个包含重复元素的数组

**参数**

- `data`: `any`: 要重复的元素
- `times`: `number`: 重复的次数，必须是非负整数

**返回值** `type(data)[]`

**示例**

```mira
repeat(0, 5) // [0, 0, 0, 0, 0]
repeat("a", 3) // ["a", "a", "a"]
```

### `fn replace(str, search, replacement)`

替换字符串中的指定子串

**参数**

- `str`: `string`: 要处理的字符串
- `search`: `string`: 要替换的子串
- `replacement`: `string`: 替换后的字符串

**返回值** `string`

**示例**

```mira
replace("foo bar foo", "foo", "baz") // "baz bar baz"
```

### `fn reverse(arr)`

返回数组的反转副本

**参数**

- `arr`: `array`: 要反转的数组

**返回值** `array`

**示例**

```mira
reverse([1, 2, 3]) // [3, 2, 1]
```

### `fn round(x)`

返回四舍五入后的整数

**参数**

- `x`: `number`: 要四舍五入的数

**返回值** `number`

### `fn sar(x, y)`

返回第一个操作数右移指定的位数

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 位数

**返回值** `number`

**示例**

```mira
sar(-8, 1) // -4
```

### `fn shl(x, y)`

返回第一个操作数左移指定的位数

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 位数

**返回值** `number`

**示例**

```mira
shl(3, 2) // 12
```

### `fn shr(x, y)`

返回第一个操作数无符号右移指定的位数

**参数**

- `x`: `number`: 第一个操作数
- `y`: `number`: 位数

**返回值** `number`

**示例**

```mira
shr(8, 1) // 4
```

### `fn sign(x)`

返回数值的符号（正数为 1，负数为 -1，零为 0）

**参数**

- `x`: `number`: 要判断符号的数

**返回值** `number`

### `fn sin(x)`

返回数值的正弦值

**参数**

- `x`: `number`: 要计算正弦的数（弧度）

**返回值** `number`

### `fn sinh(x)`

返回数值的双曲正弦值

**参数**

- `x`: `number`: 要计算双曲正弦的数

**返回值** `number`

### `fn sort(data, comparator)`

对数组中的元素进行排序，并返回排序后的结果

**参数**

- `data`: `array`: 要排序的数组
- `comparator`: `fn(a: any, b: any) -> number`: 用于比较两个元素的函数，返回一个数字，表示它们的相对顺序，默认按升序排列

**返回值** `array`

**示例**

```mira
sort(["c", "a", "b"]) // ["a", "b", "c"]
```

### `fn sort_by(data, key_fn, comparator)`

根据键函数对数组中的元素进行排序，并返回排序后的结果

**参数**

- `data`: `array`: 要排序的数组
- `key_fn`: `fn(value: any, index: number, arr: type(data)) -> any`: 用于提取排序键的函数，接受一个元素并返回其排序键
- `comparator`: `fn(a: any, b: any) -> number`: 用于比较两个排序键的函数，返回一个数字，表示它们的相对顺序，默认按升序排列

**返回值** `array`

**示例**

```mira
sort_by([(0, "x"), (2, "y"), (1, "z")], fn (item) { item[0] }) // [(0, "x"), (1, "z"), (2, "y")]
```

### `fn split(str, separator)`

将字符串拆分为子串数组

**参数**

- `str`: `string`: 要拆分的字符串
- `separator`: `string`: 分隔符

**返回值** `string[]`

**示例**

```mira
split("a,b,c", ",") // ["a", "b", "c"]
```

### `fn sqrt(x)`

返回数值的平方根

**参数**

- `x`: `number`: 要开平方的数

**返回值** `number`

### `fn starts_with(str, search)`

检查字符串是否以指定子串开头

**参数**

- `str`: `string`: 要检查的字符串
- `search`: `string`: 要匹配的子串

**返回值** `boolean`

**示例**

```mira
starts_with("mira", "mi") // true
```

### `fn sum(..values)`

返回一组数的总和

**参数**

- `..values`: `number[]`: 要计算的数值

**返回值** `number`

**示例**

```mira
sum(1, 2, 3, 4) // 10
```

### `fn tan(x)`

返回数值的正切值

**参数**

- `x`: `number`: 要计算正切的数（弧度）

**返回值** `number`

### `fn tanh(x)`

返回数值的双曲正切值

**参数**

- `x`: `number`: 要计算双曲正切的数

**返回值** `number`

### `fn to_boolean(data, fallback)`

将布尔值标准化

**参数**

- `data`: `boolean`: 要转换的数据，仅当为布尔类型时才会参与转换
- `fallback`: `any`: 当输入不是布尔值时返回的值

**返回值** `boolean | type(fallback)`

**示例**

```mira
to_boolean(true, false)   // true
to_boolean(nil, "failed") // "failed"
to_boolean(0, "failed")   // "failed"
```

### `fn to_datetime(datetime, offset, fallback)`

将数据转换为 DateTime 记录

**参数**

- `datetime`: `number | string`: 要转换的数据，默认为当前时间
- `offset`: `number`: 时区偏移量（单位：小时），默认为 0
- `fallback`: `any`: 转换失败时的返回值

**返回值** `DateTime | type(fallback)`

**示例**

```mira
to_datetime(0)
// (
//   year: 1970, month: 1, day: 1,
//   hour: 0, minute: 0, second: 0,
//   millisecond: 0,
//   dayOfWeek: 4, offset: 0
// )
```

### `fn to_iso8601(datetime, fallback)`

将数据转换为 ISO 8601 格式的字符串

**参数**

- `datetime`: `number | string`: 要转换的数据，默认为当前时间
- `fallback`: `any`: 转换失败时的返回值

**返回值** `string | type(fallback)`

**示例**

```mira
to_iso8601(0) // "1970-01-01T00:00:00.000Z"
```

### `fn to_json(data)`

将数据转换为 JSON 字符串

**参数**

- `data`: `any`: 要转换为 JSON 的数据

**返回值** `string`

**示例**

```mira
to_json([1, 2, 3]) // "[1,2,3]"
```

### `fn to_number(data, fallback)`

将数据转换为数字

**参数**

- `data`: `string | number | boolean`: 要转换的数据
- `fallback`: `any`: 转换失败时的返回值

**返回值** `number | type(fallback)`

**示例**

```mira
to_number("1.5") // 1.5
```

### `fn to_string(data, fallback)`

将数据转换为字符串

**参数**

- `data`: `any`: 要转换的数据
- `fallback`: `any`: 转换失败时的返回值

**返回值** `string | type(fallback)`

**示例**

```mira
to_string([1, 2]) // "1, 2"
```

### `fn to_timestamp(datetime, fallback)`

将数据转换为 Unix 毫秒时间戳

**参数**

- `datetime`: `number | string`: 要转换的数据，默认为当前时间
- `fallback`: `any`: 转换失败时的返回值

**返回值** `number | type(fallback)`

**示例**

```mira
to_timestamp("1970-01-01T00:00:00Z") // 0
```

### `fn trim(str)`

去除字符串两端的空白字符

**参数**

- `str`: `string`: 要处理的字符串

**返回值** `string`

**示例**

```mira
trim("  mira  ") // "mira"
```

### `fn trim_end(str)`

去除字符串结尾的空白字符

**参数**

- `str`: `string`: 要处理的字符串

**返回值** `string`

**示例**

```mira
trim_end("mira  ") // "mira"
```

### `fn trim_start(str)`

去除字符串开头的空白字符

**参数**

- `str`: `string`: 要处理的字符串

**返回值** `string`

**示例**

```mira
trim_start("  mira") // "mira"
```

### `fn trunc(x)`

返回数值的整数部分（去除小数）

**参数**

- `x`: `number`: 要取整数部分的数

**返回值** `number`

### `fn values(data)`

返回数组或记录的值列表

**参数**

- `data`: `array | record`: 要获取值的数组或记录

**返回值** `array`

**示例**

```mira
values((a: 1, b: 2)) // [1, 2]
```

### `fn with(data, ..entries)`

在数组或记录中设置多个键值对

**参数**

- `data`: `array | record`: 要设置的数组或记录
- `..entries`: `[..[number | string | (number | string)[], any][]]`: 要设置的键值对，成对出现

**返回值** `type(data)`

**示例**

```mira
with([10, 20], 2, 99, 3 ,100) // [10, 20, 99, 100]
(a: 1)::with(["b", 1], 2) // (a: 1, b: [nil, 2])
```

### `fn zip(data)`

将数组的数组/记录转换为数组/记录的数组

**参数**

- `data`: `array | record`: 要转换的数组/记录

**返回值** `(array | record)[]`

**示例**

```mira
zip((x: [1, 2], y: ["a", "b"])) // [(x: 1, y: "a"), (x: 2, y: "b")]
zip([[1, 2], ["a", "b"]]) // [[1, "a"], [2, "b"]]
```

## 模块 `matrix`

见[matrix](./10-matrix.md)。
