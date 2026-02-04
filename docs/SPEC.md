# MiraScript

## 工具链

MiraScript 的工具链包括以下几个部分：

- `mira-core`：编译器，使用 rust 编写，负责将 MiraScript 源代码编译为中间代码，并进行语法检查。
- `mira-js`：虚拟机及标准库实现，使用 TypeScript 编写，负责执行中间代码。
- `mira-py`：虚拟机及标准库实现，使用 Python 编写，负责执行中间代码。
- `mira-lsp`：语言服务器，使用 TypeScript 编写，基于 `mira-core` 和 `mira-js`，提供代码补全、语法检查、跳转定义等功能。

## 数据类型

MiraScript 支持以下数据类型：

- `number`：数字类型，使用 64 位浮点数表示。
- `string`：字符串类型，表示一个合法的 Unicode Code Point 序列，其实际编码方式由宿主环境决定，MiraScript 不提供特定于编码的字符串操作。
- `boolean`：布尔类型，包含 `true` 和 `false` 两个值。
- `nil`：空类型，`nil` 是该类型的唯一值，表示一个空值。在函数返回值中，`nil` 表示没有返回值。
- `record`：记录类型，表示一个不可变的键值对集合。记录的键必须是字符串，值可以是任意值类型。记录的键值对在创建时确定，不能修改。
- `array`：数组类型，表示一个不可变的有序值集合。数组的值可以是任意值类型。数组的元素在创建时确定，不能修改。
- `function`：函数类型，表示一个可调用的函数对象。
- `module`：模块类型，表示一个 Mirascript 模块。模块是一个不可变的键值对集合，其值可以为任意类型。
- `extern`：外部类型，来自宿主环境且无法映射为 MiraScript 原生类型的对象，MiraScript 不对其进行任何限制。外部对象可以是任意宿主环境支持的对象类型，如 JavaScript 对象、Python 对象等。MiraScript 不对外部对象的属性和方法进行限制，允许用户自由访问和操作外部对象。

### 值语义和可变性

除了 `function`、`module` 和 `extern` 外，MiraScript 的所有类型均为值类型。这意味着在进行比较时，MiraScript 会比较数据类型的值，而不是它们的引用。

除了 `extern` 类型，MiraScript 的所有类型都是不可变的。不可变性意味着一旦创建，数据的值就不能被修改。注意 `module` 类型可以包含 `extern` 类型的值，因此其深层次的值可能是可变的。

## 抽象操作

TODO:

## MiraScript 语言

MiraScript 是一种表达式语言，大部分语法结构都是表达式。MiraScript 的语法结构包括：

### 注释

MiraScript 支持单行注释和多行注释：

- 单行注释以 `//` 开头，直到行末结束。
- 多行注释以 `/*` 开头，以 `*/` 结束，可以跨越多行，不支持嵌套。

```mira
// 这是一个单行注释
/* 这是一个多行注释
   可以跨越多行 */
```

### 标识符

标识符是一个变量、函数、类型等的名称。

标识符以任意数量的 `$`、`@`，或以 `_` 或字母开头，后面可以跟任意数量的字母、数字或 `_`。标识符不能包含空格和其他特殊字符。

标识符是区分大小写的，`a` 和 `A` 是不同的标识符。

以 `$` 或 `__` 开头的标识符保留供 MiraScript 或全局环境使用，用户不应使用这些标识符。

以 `@` 开头的标识符为常量，必须使用 `const` 声明，不允许重新赋值。

### 字面量

字面量是一个固定的值，表示一个常量。MiraScript 支持以下几种字面量：

#### `nil` 字面量

表示一个空值，`nil` 是该类型的唯一值。

#### 布尔字面量

表示一个布尔值，只有两个值：`true` 和 `false`。

#### 数字字面量

表示一个数字值。

```mira
1
0.12      // 对于包含小数点的数字，其整数和小数部分均不能省略
1.0e-10   // 科学计数法
0xFF      // 十六进制数字
0o10      // 八进制数字
0b1010    // 二进制数字
1_000_000 // 数字字面量可以使用下划线分隔符

inf // 正无穷大
nan // 非数
```

##### 序数

序数是一种特殊的数字字面量，可以用于记录的键及成员访问。序数使用十进制数字表示，不能包含小数点、下划线、科学计数法和多余的前导零。序数的范围是 `0` 到 `2147483647`（`2^31 - 1`），超过该范围的整数会被视为普通数字字面量。

#### 字符串字面量

表示一个字符串值，可以使用 `'`、`"`、`` ` `` 括起来。字符串中除 `\`、`$` 和字符串使用的引号外的所有字符都可以直接使用。

```mira
'hello';
'world';
`hello
world`; // 无需使用 `\n` 转义换行符
```

使用 `\` 转义字符可以转义字符串中的特殊字符：

| 转义字符   | 描述                                                 |
| ---------- | ---------------------------------------------------- |
| `\'`       | 单引号                                               |
| `\"`       | 双引号                                               |
| `` \` ``   | 反引号                                               |
| `\\`       | 反斜杠                                               |
| `\n`       | 换行符                                               |
| `\r`       | 回车符                                               |
| `\t`       | 制表符                                               |
| `\b`       | 退格符                                               |
| `\f`       | 换页符                                               |
| `\v`       | 垂直制表符                                           |
| `\0`       | 空字符                                               |
| `\$`       | 美元符号                                             |
| `\xXX`     | ASCII 字符，`XX` 是小于 128 的十六进制数             |
| `\u{XXXX}` | Unicode 字符，`XXXX` 是合法 Unicode 码点的十六进制数 |

##### 逐字字符串

使用任意个 `@` 开始的字符串表示一个逐字字符串，逐字字符串中 `$` 以外的字符都被视为普通字符，不会被转义。使用相同数量的 `@` 结束逐字字符串。

```mira
@"verbatim string \n"@; // str 的值为 "verbatim string \\n"
@@"use `"@` in string"@@; // str 的值为 "use `\"@` in string"
```

#### 记录字面量

表示一个 `record` 类型的值。使用 `()` 括起来，记录的键值对用逗号分隔。记录的键必须是字符串，值可以是任意类型。

使用 `?:` 省略值为 `nil` 的键。

```mira
let simple_record = (key1: "value1", key2: 2, key3: true); // 简单键名必须为合法的标识符
let ordinal_record = (0: 1, 1: 2, 2: 3);                   // 或序数
let unnamed_record = ("value1", 2, true);                  // 未命名键被自动命名为 `0`、`1`、`2`
let spread_record = (key1: "new", ..simple_record, key3: false);
// 按照顺序覆盖，`spread_record` 的值为 `(key1: "value1", key2: 2, key3: false)`
let empty_record = ();                                    // 空记录
let single_record = (key1: "value1");                     // 单个键值对的记录
let special_name_record = ("name\n": "value1");           // 键名不是有效标识符
let single_unnamed_record = ("value1", );                 // 为了避免歧义，必须使用逗号
let interpolated_name_record = (`${ 1 + 2 }`: "value1");    // 键名为插值字符串的值，即 `3`
let omit_name_record = (:simple_record);                  // 键名推断为 `simple_record`
let invalid_mix = (..simple_record, "new");               // 错误，为避免歧义，未命名的键值对不能与其他构造混用
let skip_nil = (nil?: nil, no_nil?: "no_nil");            // 使用 `?:` 省略值为 `nil` 的键，值为 `(no_nil: "no_nil")`
```

使用 `.`、`[]` 操作符访问记录的属性：

```mira
let name = "Alice";
let named_record = (:name, age: 30);
debug_print(named_record.name); // 输出 "Alice"
debug_print(named_record["age"]); // 输出 "30"

let unnamed_record = (-4, 3);
debug_print(`${unnamed_record.0}, ${unnamed_record[1]}`); // 输出 "-4, 3"
```

也可以使用 JSON Object 语法书写记录字面量。此时，记录的键必须使用引号，不允许省略。

```mira
let json_record = { "key1": "value1", "key2": 2, "key3": true };
```

#### 数组字面量

表示一个 `array` 类型的值。使用 `[]` 括起来，数组的元素用逗号分隔。数组的元素可以是任意类型。

```mira
let simple_array = [1, "2", true];         // 数组的元素可以是不同类型
let empty_array = [];                      // 空数组
let range_array = [1, 2, 5..8, 4..<6];     // [1, 2, 5, 6, 7, 8, 4, 5]
let spread_array = [1, 2, ..simple_array]; // [1, 2, 1, "2", true]
```

在数组构造中，范围 `..` 和 `..<` 可以用于快速构造数字数组。此处必须指定范围的起始值和结束值。范围的含义是从起始值开始，以 1 为公差的序列。当起始值或结束值包含 `inf` 或 `nan` 时，生成的序列为空。

```mira
["4".."6"]; // [4, 5, 6]
[1..3]; // [1, 2, 3]
[1..<3]; // [1, 2]
[1.2..5.5]; // [1.2, 2.2, 3.2, 4.2, 5.2]
[1..]; // 错误，必须同时指定范围的起始值和结束值

let (start, end) = (1.2, "5.5");
[start..end]; // [1.2, 2.2, 3.2, 4.2, 5.2]
let end2 = ();
[start..end2]; // []
```

使用 `.`、`[]` 操作符访问数组的元素，使用负数索引访问数组的倒数第几个元素。当索引非整数时，会先将其转换为整数。

```mira
let array = [1, 2, 3];
debug_print(array[0]);  // 输出 "1"
debug_print(array.1);   // 输出 "2"
debug_print(array[-1]); // 输出 "3"
```

使用范围 `..`、`..<` 操作符访问数组的切片，此时范围的起始值和终止值均可省略，表示从数组的开头或结尾开始。

```mira
let array = [1, 2, 3, 4, 5];
debug_print(array[1..3]);   // 输出 "2, 3, 4"
debug_print(array[1..<3]);  // 输出 "2, 3"
debug_print(array[1..]);    // 输出 "2, 3, 4, 5"
debug_print(array[..3]);    // 输出 "1, 2, 3, 4"
debug_print(array[..<3]);   // 输出 "1, 2, 3"
debug_print(array[..]);     // 输出 "1, 2, 3, 4, 5"
debug_print(array[1..-2]);  // 输出 "2, 3, 4"
debug_print(array[4..<-2]); // 输出 ""
```

### 范围

范围表示一个值的范围。范围的语法为 `start..end` 或 `start..<end`，其中 `start` 和 `end` 是一个加法运算表达式，表示范围的起始值和结束值。

其中 `..` 表示闭区间，`..<` 表示右开区间。

范围不是一个完整的语法结构，其含义依赖于上下文。范围可以用于数组初始化、数组的切片、`for` 语句、模式匹配等场景。

依据上下文，`start` 和 `end` 的类型需求也不相同，有时也可以/必须省略。

`..` 和 `..<` 操作符的优先级介于加法运算（二元 `+`、`-`）和模式匹配（`is`）之间。

### 类型转换

在需要时，MiraScript 会对值的类型进行转换，规则如下：

#### 转换为 `string`

基本上，所有 MiraScript 值都可以成功转换为 `string`。其中，`nil` 将被转换为空字符串；`record` 和 `array` 的元素之间将使用 `', '` 连接。

#### 转换为 `number`

MiraScript 中以下值支持转换为 `number`：

- `true`：转换为 `1`；
- `false`：转换为 `0`；
- `string`：除了表示数字的字符串外，`"nan"` `"NaN"` `"[+-]?inf"` `"[+-]Infinity"` 也可以被转换为 `number`。

除此之外的其他值，包括不表示数字的字符串、空字符串，在转换过程中均会产生异常。

#### 转换为 `boolean`

MiraScript 不支持其他类型转换为 `boolean`。所有转换的尝试均产生异常。

### 表达式

MiraScript 的表达式是一个值的计算，表达式可以是一个值、一个变量、一个函数调用、一个操作符等。MiraScript 的表达式包括：

#### 基础表达式

基础表达式表示对值的计算，基础表达式包括字面量、插值字符串、变量、函数调用、操作符等。

```mira
let x = 1;       // 变量 x 的值为 1
let r = (1, 2);  // 记录 r 的值为 (1, 2)
x + 2;           // 表达式的值为 3
r.0^2 + r.1^2;   // 表达式的值为 5
x * (r.0 + r.1); // 使用括号改变运算顺序，表达式的值为 6
```

##### 插值字符串

使用 `$` 可以在字符串中插入变量或块表达式的值，对于**逐字字符串**，使用与开始字符串 `@` 相同数量的 `$` 表示插值：

```mira
let name = "world";
debug_print("hello, $name"); // 输出 "hello, world"
debug_print(@"hello, $name"@); // 输出 "hello, world"
debug_print(@@"hello, $name: $$name"@@); // 输出 "hello, $name: world"

let a = 1;
let b = 2;
debug_print("the sum of ${a} and ${b} is $(a + b:.1)"); // 输出 "the sum of 1 and 2 is 3.0"
```

插值字符串的语法是 `<prefix> ( <expression> : <format> )`，其中 `<prefix>` 是指定数量的 `$`，`<expression>` 是一个表达式。插值字符串的值为 `<expression>` 的值。

使用默认格式时，`: <format>` 可以省略。

当 `<expression>` 为一个**标识符**或**块表达式**时，括号 `()` 可以省略。

##### 优先级与结合性

| 优先级 | 分类     | 结合性   | 运算符               | 注释                               |
| ------ | -------- | -------- | -------------------- | ---------------------------------- |
| 最高   | 分组     | /        | 分组 `(x)`           | `x` 可以是任意表达式               |
|        |          |          | 表达式块 `{}`        |                                    |
|        | 后缀操作 | 从左到右 | 成员访问 `x.y`       | `y` 必须是标识符或序数             |
|        |          |          | 成员访问 `x[y]`      | `y` 可以是任意表达式               |
|        |          |          | 切片 `x[y]`          | `y` 是一个范围                     |
|        |          |          | 函数调用 `x(y)`      | `y` 是 `,` 分隔的参数列表          |
|        |          |          | 扩展调用 `x::m(y)`   | `m` 是仅包含分组和成员访问的表达式 |
|        |          |          | 模板调用 `x"y"`      | `"y"` 是一个插值字符串             |
|        |          |          | 非空断言 `x!`        |                                    |
|        |          |          | `type(x)`            | 结果为 `x` 类型的字符串            |
|        | 乘方运算 | 从右到左 | 乘方 `x^y`           | 操作数为 `number` 类型             |
|        | 前缀操作 | /        | 逻辑非 `!x`          | 结果为 `boolean` 类型              |
|        |          |          | 一元加 `+x`          | 结果为 `number` 类型               |
|        |          |          | 一元减 `-x`          |                                    |
|        | 乘法运算 | 从左到右 | 乘法 `x * y`         |                                    |
|        |          |          | 除法 `x / y`         |                                    |
|        |          |          | 取余 `x % y`         |                                    |
|        | 加法运算 | 从左到右 | 加法 `x + y`         |                                    |
|        |          |          | 减法 `x - y`         |                                    |
|        | 模式匹配 | 从左到右 | 模式匹配 `x is y`    | `x` 是一个表达式，`y` 是一个模式   |
|        | 关系运算 | 从左到右 | 包含 `x in y`        |                                    |
|        |          |          | 大于 `x > y`         | 操作数为 `number` 或 `string` 类型 |
|        |          |          | 大于等于 `x >= y`    |                                    |
|        |          |          | 小于 `x < y`         |                                    |
|        |          |          | 小于等于 `x <= y`    |                                    |
|        | 相等运算 | 从左到右 | 相等 `x == y`        | 操作数不进行隐式类型转换           |
|        |          |          | 不相等 `x != y`      | 等价于 `!(x == y)`                 |
|        |          |          | 近似相等 `x =~ y`    | 操作数为 `number` 或 `string` 类型 |
|        |          |          | 不近似相等 `x !~ y`  | 等价于 `!(x =~ y)`                 |
|        | 逻辑与   | 从左到右 | 逻辑与 `x && y`      | 操作数为 `boolean` 类型            |
|        | 逻辑或   | 从左到右 | 逻辑或 `x \|\| y`    |                                    |
|        | 空合并   | 从左到右 | 空合并 `x ?? y`      |                                    |
| 最低   | 条件运算 | 从右到左 | 条件运算 `c ? x : y` | 条件为 `boolean` 类型              |

##### 空安全

MiraScript 默认的访问语义是空安全的，成员访问操作符 `.` 和 `[]` 的左操作数如果为 `nil`，或要索引的属性不存在，则返回 `nil`，而不是抛出异常。

```mira
let x = (1,);
let y = x.2; // y 的值为 nil
let z = x.0.non_existent; // z 的值为 nil
let w = x.1.2; // w 的值为 nil
let r = x.fun(); // r 的值为 nil
```

使用 `!` 操作符进行非空断言，`!` 操作符的左操作数如果为 `nil`，则抛出 `NilError` 异常。

```mira
let x = (1);
let y = x.0!; // y 的值为 1
let z = x.2!; // 抛出 NilError 异常
let w = x.1.2!; // 抛出 NilError 异常
let w = x.1!.2; // 抛出 NilError 异常
let r = x.fun!(); // 抛出 NilError 异常
```

使用空合并操作符 `??` 可以在左操作数为 `nil` 时返回右操作数的值。

```mira
let x = (1);
let y = x.2 ?? 0; // y 的值为 0
```

##### 布尔运算

MiraScript 中使用 `&&`、`||` 和 `!` 运算符进行布尔运算，参与运算的操作数必须为 `boolean` 类型。

也可以使用 `and`、`or` 和 `not` 关键字作为布尔运算符的替代写法。

##### 短路求值

MiraScript 的逻辑运算符支持短路求值，`&&`、`||` 和 `??` 运算符的右操作数只有在左操作数为 `true`、`false` 或 `nil` 时才会被求值。

```mira
let x = false;
let y = true;
let z = 0;
let and_result = x && y;     // x 为 false，y 不会被求值，and_result 的值为 false
let or_result = y || z;      // y 为 true，z 不会被求值，or_result 的值为 true
let nil_coalescing = x ?? z; // y 不为 nil，z 不会被求值，nil_coalescing 的值为 false
```

MiraScript 的链式调用也支持短路求值，在函数调用中，当操作数不是标识符且值为 `nil` 时，参数不会被求值，调用的返回值为 `nil`。

```mira
fn f() { nil }
fn g() { 0 }
let n = nil;
fn x { it }

f()(x()); // f() 为 nil，x 不会被调用，表达式的值为 nil
g()(x()); // x 被调用，尝试将 g() 转换为 function 时抛出 TypeError 异常

n(x());   // x 被调用，尝试将 n 转换为 function 时抛出 TypeError 异常
n!(x());  // 对 n 进行非空断言时抛出 NilError 异常，x 不会被调用
(n)(x()); // (n) 为 nil，x 不会被调用，表达式的值为 nil
```

##### 关系运算

使用 `in` 操作符测试记录是否包含某个键，或数组是否包含某个值。

```mira
let x = (nil, );
let y = 0 in x; // y 的值为 true;
let z = 1 in x; // z 的值为 false;

let a = ["hello", "world"];
let b = "hello" in a; // b 的值为 true;
```

使用 `>`、`<`、`>=`、`<=` 运算符比较两个值的大小。MiraScript 会尝试将操作数转换为 `number` 或 `string` 类型进行比较。

```mira
let x = 1;
let y = "2";
let z = nil;
let w = ();
x > y; // y 转换为 number 类型 2，结果为 false
y <= z; // z 转换为 string 类型 ""，结果为 false
x >= w; // 尝试将 w 转换为 number 类型 nan，结果为 false
z < w; // z、w 转换为 number 类型均为 nan，结果为 false
```

##### 相等运算

使用 `==`、`!=` 运算符比较两个值是否相等。MiraScript 不会对操作数进行隐式类型转换。

```mira
1 == "1"; // false
+0 == -0; // true
nan == nan; // false
```

在比较记录和数组时，MiraScript 会比较它们的键值对或元素的值是否为相同值。

```mira
(nan,) == (nan,); // true
(1, 2) == (1, 2); // true
(1, 2) == (2, 1); // false
[1, 2] == [1, 2]; // true
[1, 2] == [2, 1]; // false
```

在比较 `extern` 和 `function` 时，MiraScript 会比较它们的引用是否相同。

```mira
let x = fn {};
let y = fn {};
x == y; // false
```

使用 `=~`、`!~` 运算符比较两个值是否近似相等。MiraScript 会尝试将操作数转换为 `number` 或 `string` 进行比较。

对于 `number` 类型，当两个操作数的相对误差**或**绝对误差小于 `1e-15` 时，返回 `true`，否则返回 `false`。

当其中一个操作数为 `nan` 时，`=~` 运算符始终返回 `false`。

对于 `string` 类型，`=~` 运算符进行大小写不敏感的[正规化（NFC）](https://unicode.org/reports/tr15/)比较。

```mira
1 =~ 1.0000000000000002; // true
"1" =~ 1; // true
"1" =~ "1.0000000000000002"; // false
"A" =~ "a"; // true
"a" =~ nan; // "a" 转换为 number 类型 nan，返回 false
(1, ) =~ (1.0000000000000002, ); // 两侧转为 number 类型均为 nan，返回 false
```

##### 函数调用

参数列表中可使用 `..` 操作符将数组展开为多个参数。

```mira
fn add(x, y) {
  x + y
};

let result = add(1, 2);
let array = [3, 4];
let result2 = add(..array);
```

##### 扩展调用运算符

扩展调用运算符 `::` 将左操作数作为右侧函数调用的第一个参数，用以简化函数调用的语法。

```mira
[1, 2, 3]::filter(fn { it > 1 })::(fn { `The array is: $it` })()::debug_print();
// 相当于
// debug_print(
//   (fn { `The array is: $it` })(
//     filter(
//       [1, 2, 3],
//       fn { it > 1 }
//     )
//   )
// );
```

> ###### 设计备忘
>
> - 占位符
>
>   替代语法使用占位符以允许在扩展调用中使用任意表达式，如：
>
>   ```mira
>   [1, 2, 3]::filter(fn { it > 1 })::[`The array is: $_`]::debug_print();
>   ```
>
>   考虑到扩展调用有较高优先级，可以简单与其他元素组合构造表达式，且无需引入额外的语法糖，决定不使用占位符语法。当前语法下的替代写法：
>
>   ```mira
>   debug_print(`The array is: ${[1, 2, 3]::filter(fn { it > 1 })}`);
>   ```
>
> - 管道运算
>
>   另一种替代语法是使用管道运算符 `|>` `<|` 进行链式调用，如：
>
>   ```mira
>   [1, 2, 3] |> filter(fn { it > 1 }) |> fn { `The array is: $it` } |> debug_print;
>   ```
>
>   其优先级太低，与其他元素组合构造表达式时需要使用括号，引入了较多语法噪音。比较：
>
>   ```mira
>   len(arr) > 10;                  // 传统写法
>   arr::len() > 10;                // 使用扩展调用
>   (arr |> len()) > 10;            // 使用管道运算，添加括号
>   arr |> len() |> fn { it > 10 }; // 使用管道运算，通过函数表达式移除括号
>   arr |> len() |> _ > 10;         // 管道运算的占位符语法
>   ```

##### 模板调用

当函数调用的参数是一个模板字符串时，会进行模板调用。以模板字符数组作为第一个参数，插值表达式个格式作为其余参数调用函数。

```mira
foo"Hello $(name:fmt)";
// 相当于：
foo(["Hello ", ""], (name, "fmt"));
```

#### 块表达式

块表达式用于定义一个代码块，块表达式的语法为 `{ <statements> [<expression>] }`。其中 `<statements>` 是一系列语句。

当可选的 `<expression>` 存在时，块表达式的值为 `<expression>` 的值；当 `<expression>` 不存在时，块表达式的值为 `nil`。

```mira
let x = { }; // x 的值为 nil
let y = {
  let a = 1;
  let b = 2;
  a + b
}; // y 的值为 3
let z = {
  let mut a = 1;
  a += y;
}; // z 的值为 nil
```

#### `if` 表达式

`if` 表达式用于条件判断，语法为 `if <condition> <then_expression> [else <else_expression>]`。其中 `<condition>` 是一个条件表达式，无需使用括号括起来；`<then_expression>` 是条件为 `true` 时执行的表达式，必须为块表达式；`<else_expression>` 是条件为 `false` 时执行的表达式，必须为 `if` 表达式或块表达式；条件为其他值时会抛出异常。

`if` 表达式的值为 `<then_expression>` 或 `<else_expression>` 的值。当条件为 `false` 且不存在 `<else_expression>` 时，`if` 表达式的值为 `nil`。

```mira
let x = 1;
let y = if x > 0 {
  "positive"
} else if x < 0 {
  "negative"
} else {
  "zero"
}; // y 的值为 "positive"
```

#### `match` 表达式

`match` 表达式用于模式匹配，语法为 `match <test> { [case <pattern> [if <guard>] <expression>]... }`。其中 `<test>` 是一个表达式；`<pattern>` 是一个模式；`<guard>` 是一个表达式；`<expression>` 是一个块表达式。

`match` 表达式的值为第一个匹配成功的 `<expression>` 的值。当没有匹配成功的模式时，表达式的值为 `nil`。

```mira
let x = 1;
let y = match x {
  case 1 { "one" }
  case 2 { "two" }
  case 3 { "three" }
  case x if x > 0 { "positive" }
  case _ { "other" }
}; // y 的值为 "one"
let z = match x { }; // z 的值为 nil
```

#### `for`/`while`/`loop` 表达式

`for`/`while`/`loop` 表达式用于循环，语法为 `for <pattern> in <expression_or_range> { <statements> } [else <else_expression>]`、`while <condition> { <statements> } [else <else_expression>]` 和 `loop { <statements> }`。

`for` 表达式用于遍历一个数组、记录或范围，`while` 表达式用于循环执行一段代码，`loop` 表达式用于无限循环。

使用 `continue` 语句可以跳过当前循环的剩余部分，继续执行下一次循环；使用 `break` 语句可以退出循环，并返回一个值。

`for`/`while`/`loop` 表达式的值为退出循环时 `break` 语句的值。当循环正常结束时，表达式的值为 `<else_expression>` 的值。当没有 `else_expression` 时，表达式的值为 `nil`。

```mira
let array = [1, 2, 3];
let mut sum = 0;
for i in array {
  sum += i;
}
// sum 的值为 6

let record = ("can", "you", "find", "me");
let found = for key in record {
  if record[key] == "me" {
    break key;
  }
} else {
  "not found"
}; // found 的值为 3

let mut count = 0;
while count < 5 {
  count += 1;
} else {
  "done"
} // count 的值为 5，表达式的值为 "done"

let mut i = 0;
loop {
  i += 1;
  if i == 5 {
    break i;
  }
} // i 的值为 5，表达式的值为 5
```

#### 函数表达式

函数表达式用于定义一个函数，函数表达式的语法为 `fn (<parameters>) <body>`。其中 `<parameters>` 是函数的参数列表，可以是一个或多个参数，参数之间用逗号分隔；`<body>` 是函数的函数体，是一个块表达式，表示函数的实现，该表达式的值即为函数的返回值。

```mira
let add = fn (x, y) {
  x + y
}; // add 是一个 `function` 类型的值
```

当函数有且仅有一个参数时，参数列表可以省略，此时该参数的名称为 `it`。

```mira
let add_one = fn {
  it + 1
};
```

### 模式

模式用于匹配数据结构的形状和内容，在 `let` 语句、赋值语句、`match` 表达式、`for` 表达式、`is` 运算符等场景中使用。

MiraScript 支持以下几种模式：

#### 字面量模式

字面量模式用于匹配字面量的值。字面量模式的语法为 `<literal>`，其中 `<literal>` 是 `nil`、`true`、`false`、`nan`、`inf`、数字字面量或不包含插值的字符串字面量，及它们与前缀运算符 `+`、`-` 结合的结果。

与 `==` 运算符不同，字面量模式使用相同值语义进行匹配。

```mira
fn is_nan { it is nan }
```

#### 常量模式

与字面量模式类似，常量模式用于匹配常量的值。常量模式的语法为 `<constant>`，其中 `<constant>` 是一个以 `@` 开头的标识符名称。

```mira
fn is_pi {
  const @pi = PI;
  it is @pi
}
```

#### 关系模式

关系模式用于匹配关系运算的结果。关系模式的语法为 `<relation> <value>`，其中 `<relation>` 是 `>`、`<`、`<=`、`==`、`!=`、`=~`、`!~` 运算符，`<value>` 是一个字面量模式或常量模式。

关系模式相当于对匹配到的值进行 `type(<captured>) == type(<value>) && <captured> <relation> <value>` 的判断，当该判断返回 `false` 时，匹配失败。

```mira
fn gpa {
  match it {
    case >= 3.5 { "A" }
    case >= 3.0 { "B" }
    case >= 2.5 { "C" }
    case >= 2.0 { "D" }
    case _ { "F" }
  }
}
```

#### 范围模式

范围模式用于匹配数字范围。范围模式的语法为 `<start>..<end>` 或 `<start>..<<end>`，其中 `<start>` 和 `<end>` 是数字字面量模式或常量模式。

范围模式相当于对匹配到的值进行 `<captured> >= <start>` 和 `<captured> <= <end>` / `<captured> < <end>` 的判断，当该判断返回 `false` 时，匹配失败。

范围模式中不会进行隐式类型转换，只有当 `<captured>` 为 `number` 时，才会进行后续的测试，否则匹配失败。

```mira
fn season {
  match it {
    case 1..3 { "Spring" }
    case 4..6 { "Summer" }
    case 7..9 { "Fall" }
    case 10..12 { "Winter" }
    case _ { "Unknown" }
  }
}
```

#### 变量模式

变量模式可以匹配任意值，并将匹配成功的值绑定到变量上。变量模式的语法为 `[mut] <variable>`，其中 `<variable>` 是一个不以 `@` 开头的标识符名称。声明新变量时，可以使用 `mut` 关键字表示该变量是可变的。

```mira
let x = 1; // 变量 x 的值为 1
let mut y = 2; // 变量 y 的值为 2

if x is (mut z and not nan) {
  z += 1; // z 的值为 2
  debug_print(z);
} else {
  debug_print("not a number");
}
```

#### 弃元模式

与变量模式类似，弃元模式用于匹配任意值，但不绑定该值。弃元模式的语法为 `_`，可以与其他模式结合使用。

```mira
let x = 1;
let y = 2;
_ = x + y; // 匹配 x + y 的值，但不绑定该值
```

#### 记录模式

记录模式用于匹配记录。记录模式的语法为 `([<sub_pattern>]...)`。

```mira
(1, 2, 3) is (); // 匹配成功
[1, 2, 3] is (); // 匹配失败
"string" is ();  // 匹配失败
```

`<sub_pattern>` 是记录模式的子模式，用于进一步匹配记录的键值对，包含以下几种形式：

- 具名模式

  具名模式用于匹配记录的键值对。具名模式的语法为 `<key>: <pattern>`。其中 `<key>` 是一个标识符、序数，或由 `[]` 括起来的字面量模式，表示记录的键；`<pattern>` 是一个模式，匹配记录的值。

  可以使用 `?:` 语法表示该模式是可选的，此时对不存在的键会匹配到 `nil`；此时对值为 `nil` 的键会匹配失败。

  ```mira
  let record = (key1: "value1", key2: 2, key3: true);
  record is (key1: "value1", key2: 2); // 匹配成功
  record is (no_exist: _); // 匹配失败
  record is (no_exist?: v); // 匹配成功，v 的值为 nil
  ```

- 略名模式

  略名模式用于简化具名模式的书写。略名模式的语法为 `:<pattern>` 或 `?:<pattern>`。其中 `<pattern>` 必须是一个变量模式，用以推断记录的键名。

  ```mira
  let record = (key1: "value1", key2: 2, key3: true);
  record is (key1: "value1", :mut key2, ?:no_exist); // 匹配成功，key2 的值为 2，no_exist 的值为 nil
  record is (:no_exist); // 匹配失败
  ```

- 展开模式

  展开模式用于匹配记录剩余键值对。展开模式的语法为 `..<pattern>`，表示匹配记录的剩余所有键值对。

  展开模式必须是记录模式的最后一个子模式，且只能出现一次。展开模式的值是一个记录，包含匹配成功的键值对。

  与数组模式不同，由于记录模式的匹配并不穷尽，因此单独的 `..` 是无意义的匹配，必须在 `..` 后添加一个模式进行后续匹配。

  ```mira
  let record = (key1: "value1", key2: 2, key3: true);
  record is (key1: "value1", ..rest1); // 匹配成功，rest1 的值为 (key2: 2, key3: true)

  let unnamed_record = (1, 2, 3);
  unnamed_record is (1, ..rest2); // 匹配成功，注意 rest2 的值为 (1: 2, 2: 3)

  record is (..)        // 语法错误
  record is (key1, ..) // 语法错误
  ```

- 未命名模式

  未命名模式用于匹配记录的序数键值对。未命名模式的语法为 `<pattern>`，其中 `<pattern>` 是一个模式，匹配记录的值。

  与记录字面量语法类似，为了避免歧义，记录模式中未命名模式必须出现在最前。

  ```mira
  let record = (1, 2, 3);
  record is (1, _); // 匹配成功，相当于 (0: 1, 1: _)
  ```

#### 数组模式

数组模式用于匹配数组。数组模式的语法为 `[ [<sub_pattern>]... ]`。

```mira
[1, 2, 3] is [..]; // 匹配成功
(1, 2, 3) is [..]; // 匹配失败
```

`<sub_pattern>` 是数组模式的子模式，用于进一步匹配数组的元素，包含以下几种形式：

- 元素模式

  元素模式用于匹配数组的元素。元素模式的语法为 `<pattern>`，其中 `<pattern>` 是一个模式，匹配数组的值。

  为了避免与数组的范围初始化混淆，当该模式为范围模式时，必须添加括号。

  ```mira
  let array = [1, 2, 3];
  array is [1, x, y];       // 匹配成功，x 的值为 2，y 的值为 3
  array is [x, y, z, w];    // 匹配失败，元素数量不足
  array is [];              // 匹配失败，元素数量过多
  array is [(1..10), 2, _]; // 匹配成功，范围模式须添加括号
  ```

- 展开模式

  展开模式用于匹配数组剩余元素。展开模式的语法为 `.. [<pattern>]`，表示匹配数组的所有元素。

  在一个数组模式中，展开模式只能出现一次。展开模式的值是一个数组，包含匹配成功的元素。

  ```mira
  let array = [1, 2, 3];
  array is [1, ..rest];       // 匹配成功，rest 的值为 [2, 3]
  array is [..rest, 1, 2, 3]; // 匹配成功，rest 的值为 []
  array is [_, .., _, _, _];  // 匹配失败，元素数量不足
  array is [.., 2, ..];       // 语法错误，展开模式只能出现一次
  ```

#### 逻辑模式

可以使用 `not`、`and`、`or` 关键字对模式进行组合，形成更复杂的模式。

```mira
fn not_nil { it is not nil }

fn is_on_axis { it is (_, 0) or (0, _) }

fn discount {
  match it {
    case (items: > 100) or (cost: > 500) { 0.2 }
    case (items: > 50) or (cost: > 200) { 0.15 }
    case (items: > 10) or (cost: > 100) { 0.1 }
    case _ { 0 }
  }
}
```

逻辑模式不会进行短路求值，不论匹配成功与否，后续的匹配都会继续进行。

```mira
let value = [1, 2, 3];
let matched1 = value is [x, y, 5] and [0, 0, z];
// 即使 `and` 模式第一个子模式匹配失败，第二个子模式依旧会继续匹配，x 的值为 1，y 的值为 2，z 的值为 3，matched1 为 false
let matched2 = value is [a, b, 3] or [1, 2, c];
// 即使 `or` 模式第一个子模式匹配成功，第二个子模式依旧会继续匹配，a 的值为 1，b 的值为 2，c 的值为 3，matched2 为 true
```

### 语句

MiraScript 的语句一般分号 `;` 结尾。MiraScript 的语句包括：

#### `const` 语句

定义一个常量。

`const` 语句的语法为：

```mira
const <constant> = <expression>;
```

其中 `<constant>` 是一个以 `@` 开头的标识符名称，表示常量的名称；`<expression>` 是一个表达式，表示常量的值。

#### `let` 语句

通过指定的模式定义一系列新变量。

`let` 语句的语法为：

```mira
let <pattern> = <expression>;
```

其中 `<pattern>` 是一个模式，可以是一个变量名、一个记录、一个数组等，`<expression>` 是一个表达式。let 语句的作用是将 `<expression>` 的值与 `<pattern>` 进行匹配，并将匹配成功的值与 `<pattern>` 的变量进行绑定。

```mira
let x = 1; // 初始化变量 x 的值为 1
let mut y = "hello"; // 初始化变量 y 的值为 "hello"，可以对 y 进行赋值
let (a, mut b) = (1, 2); // 记录模式，变量的可变性可以分别设置
let [first, _, ..mut rest] = [1, 2, 3, 4]; // 数组模式，first 初始化为 1，rest 初始化为 [3, 4]
```

模式匹配失败时，`let` 语句不会产生异常。

#### 赋值语句

赋值语句修改变量的值；或对 `extern` 对象的属性进行赋值。

赋值语句有以下几种形式：

- ```mira
  <pattern> = <expression>;
  ```

  与 let 语句类似，对模式中的变量重新赋值。模式中的变量必须已经声明且为 `mut`，此时的模式不能包含 `mut` 关键字。

- ```mira
  <variable> <compound_assignment> <expression>;
  ```

  修改单个变量的值，`<compound_assignment>` 是 `=` 运算符或复合赋值运算符，如 `+=`、`-=`、`*=`、`/=` 等。

- ```mira
  <extern>.<property_id> <compound_assignment> <expression>;
  <extern>[<property_expr>] <compound_assignment> <expression>;
  ```
  对 `extern` 对象的属性进行赋值，其中 `<extern>` 是一个求值结果为 `extern` 的表达式；`<property_id>` 是一个属性名;`<property_expr>` 是一个表达式，表示属性的索引，其求值结果将转换为字符串。

```mira
let mut x = 1; // 绑定变量 x 的值为 1
x += 2; // 重新绑定变量 x 的值为 3
(x, _) = ("hello", "world"); // 重新绑定变量 x 的值为 "hello"

ex.foo = 1; // 对 extern 对象 ex 的属性 "foo" 赋值
ex[1 + 2] += 1; // 对 extern 对象 ex 的属性 "3" 复合赋值
```

当 `<extern>` 表达式的求值结果不是 `extern` 对象或 `nil` 时，赋值语句会抛出 `TypeError` 异常。

#### 表达式语句

表达式语句用于执行一个表达式，并忽略其返回值。表达式语句的语法为 `<expression>;`，其中 `<expression>` 是一个表达式。

当表达式以 `}` 结尾时，表达式语句的 `;` 须省略。

#### 函数声明语句

函数声明语句用于定义一个函数。与函数表达式类似，函数声明的语法为

```mira
fn <name>(<parameters>) <body>
```

其中 `<name>` 是函数的名称；其他部分与函数表达式相同。

```mira
fn add(x, y) {
  x + y
}

fn add_one {
  it + 1
}
```

#### 控制流语句

控制流语句用于控制程序的执行流程。MiraScript 支持以下控制流语句：

- `return` 语句

  ```mira
  return <expression>;
  ```

  用于从函数或脚本中返回一个值。其中 `<expression>` 是一个可选的表达式，如果省略，则返回 `nil`。

  ```mira
  fn add(x, y) {
    return x + y;
  }
  ```

- `break` 语句

  ```mira
  break <expression>;
  ```

  用于跳出循环。其中 `<expression>` 是一个可选的表达式，作为循环表达式的值，如果省略，则循环表达式的求值结果为 `nil`。

  ```mira
  let mut i = 0;
  let result = while i < 10 {
    i += 1;
    if i == 5 {
      break i; // 跳出循环，返回 5
    }
  };
  debug_print(result); // 输出 5
  ```

- `continue` 语句

  ```mira
  continue;
  ```

  用于跳过当前循环的剩余部分，继续下一次循环。

  ```mira
  let mut i = 0;
  while i < 10 {
    i += 1;
    if i % 2 == 0 {
      continue; // 跳过偶数
    }
    debug_print(i); // 输出奇数
  }
  ```

#### `mod` 语句

定义一个模块。

`mod` 语句的语法为：

```mira
mod <name> {
  <statements>
}
```

其中 `<name>` 是模块的名称；`<statements>` 是模块内的语句。

在模块内的 `let` 语句、`const` 语句、函数声明语句和 `mod` 语句前添加 `pub` 关键字，可以将其导出为模块的公共成员。

```mira
mod math {
  pub const @pi = 3.14159;
  pub fn add(x, y) {
    x + y
  }
}
```

通过成员访问操作符访问模块的成员时，始终会获取到模块成员的当前值。

```mira
mod counter {
  pub let mut value = 0;
  pub fn increment() {
    value += 1;
  }
}

debug_print(counter.value); // 输出 0
counter.increment();
debug_print(counter.value); // 输出 1
```

#### 空语句

空语句用于占位，语法为 `;`。

## 字节码

TODO:

## 标准库

MiraScript 的标准库包含了一些常用的函数和模块，方便用户在编写脚本时使用。

TODO:
