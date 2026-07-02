---
name: mirascript-code
description: Guide AI to write correct, idiomatic MiraScript code
license: MIT
requires:
  - mirascript-cli
---

# MiraScript 编码技能

MiraScript 是一门**表达式优先**的动态类型脚本语言，设计用于嵌入到其他应用程序中。其语法风格接近 Rust，但不需要类型标注，且具有空安全机制。

## When to use

当需要编写、修改或审查 MiraScript 代码（`.mira` 文件）时使用本技能。本技能依赖 **mirascript-cli** 技能——生成代码后必须通过 CLI 进行验证和迭代。

---

## 核心语法规则

### 语句与分号

所有语句必须以分号 `;` 结尾，除了**函数体/块的最后一个表达式**（它是返回值，不加分号）：

```mira
let x = 10;
let y = x * 2;
debug_print(y);
```

### 注释

```mira
// 单行注释
/* 多行注释（不可嵌套） */
```

---

## 变量与常量

```mira
// 不可变变量（默认）
let name = "Alice";

// 可变变量（仅在需要重新赋值时使用）
let mut count = 0;
count += 1;

// 常量（@ 前缀，编译期已知）
const @MAX = 100;
```

**编码规范（Clippy 风格）**：

- 优先使用 `let`，仅当变量需要重新赋值时才使用 `let mut`
- 在循环中累积结果时必须使用 `let mut`

---

## 函数

### 声明与返回值

```mira
// 多参数函数——最后一个表达式自动作为返回值
fn add(x, y) {
  x + y
}

// 单参数简写——省略参数名和圆括号，使用 it
fn double { it * 2 }
fn square { it^2 }
```

**编码规范（Clippy 风格）**：

- **优先使用隐式 return**：函数体最后一个表达式不加分号即为返回值，不要使用 `return` 关键字结尾
- 仅在**提前返回**时使用 `return` 语句

```mira
// ✅ 正确：隐式返回
fn classify(n) {
  if n > 0 {
    return "正数";   // 提前返回——允许使用 return
  }
  if n < 0 {
    return "负数";
  }
  "零"              // 最后一个表达式，不加分号
}

// ❌ 避免：不必要地在末尾使用 return
fn bad(x) {
  return x * 2;    // 应改为 x * 2（去掉 return 和分号）
}
```

### 函数表达式（匿名函数）

```mira
let multiply = fn (x, y) { x * y };
let cube = fn { it^3 };
```

### 展开参数

```mira
fn sum_all(first, ..rest) {
  // rest 是数组
}

let args = [3, 4];
add(..args);  // 展开数组为参数
```

---

## 控制流

### if 表达式

`if` 是表达式，有返回值；条件必须为布尔值：

```mira
let label = if score >= 60 { "及格" } else { "不及格" };
```

### match 表达式

```mira
match value {
  case nil    { "空值" }
  case true   { "真" }
  case 0      { "零" }
  case > 0    { "正数" }
  case 0..<60 { "不及格" }
  case [head, ..tail] { "数组" }
  case (name: n, age: a) { "记录，姓名 $n" }
  case _      { "其他" }
}
```

`match` 未匹配任何分支时返回 `nil`。

### 循环

```mira
// for 遍历范围、数组、记录
for i in 1..5 { debug_print(i); }
for item in arr { process(item); }

// while 条件循环
while count > 0 { count -= 1; }

// loop 无限循环，break 带返回值
let result = loop {
  if done { break value; }
};
```

---

## 数组

```mira
let arr = [1, 2, 3];
let range = [1..10];      // 包含 10
let half = [0..<10];      // 不含 10
let merged = [..a, ..b];  // 展开合并

// 访问
arr[0]    // 第一个
arr[-1]   // 最后一个
arr[1..3] // 切片 [2, 3, 4]

// "修改"（创建新数组）
let updated = with(arr, 0, 99);      // 替换索引 0
let appended = [..arr, 4];           // 追加元素
```

数组是**不可变**的，所有操作都返回新数组。

---

## 记录

```mira
let person = (name: "小明", age: 20);
let point = (10, 20);           // 未命名键，自动编号 0, 1
let short = (:name, :age);      // 键名与变量同名时简写

// 访问
person.name
person["name"]
point.0

// "修改"（展开构造新记录）
let updated = (..person, age: 21);
```

记录是**不可变**的，所有操作都返回新记录。

---

## 字符串

MiraScript **不支持用 `+` 拼接字符串**，`+` 是数值加法运算符。使用字符串插值：

```mira
let name = "小明";
let age = 20;

"你好，$name"           // 插入变量
"年龄：$(age + 1)"      // 插入表达式
"等级：${ if age >= 18 { "成年" } else { "未成年" } }"  // 插入块
```

逐字字符串（不转义、使用与 `@` 相等数量的 `$` 插值）：

```mira
@"C:\Users\name"@      // 单个 @ 对
@@"含有 @"..."@ 的文本"@@
```

---

## 扩展调用与链式管道

`::` 将左侧值作为右侧函数的第一个参数，适合构建数据处理管道：

```mira
// value::func(args)  等价于  func(value, args)

let result = data
  ::filter(fn { it > 0 })
  ::map(fn { it * 2 })
  ::sort();
```

**编码规范**：复杂的嵌套调用优先改写为 `::` 链式风格，提升可读性。

---

## 空安全

```mira
// 访问不存在的属性返回 nil，不报错
record.missing_key      // nil
nil.any.chain.access    // nil

// 空合并：左侧为 nil 时取右侧默认值
let port = config.port ?? 8080;

// 非空断言：确认不为 nil，否则抛出异常（谨慎使用）
let value = maybe_nil!;
```

**编码规范**：优先使用 `??` 提供默认值，仅在确定非空时使用 `!`。

---

## 模式匹配速查

| 模式     | 示例                     | 说明         |
| -------- | ------------------------ | ------------ |
| 字面量   | `case 42 {}`             | 精确匹配     |
| 弃元     | `case _ {}`              | 匹配任意值   |
| 变量绑定 | `case x {}`              | 捕获值到 `x` |
| 关系     | `case > 0 {}`            | 比较运算     |
| 范围     | `case 1..10 {}`          | 闭区间       |
| 半开范围 | `case 0..<5 {}`          | 不含上界     |
| 数组     | `case [a, b] {}`         | 精确元素数   |
| 数组展开 | `case [head, ..tail] {}` | 首元素+其余  |
| 记录     | `case (x: val) {}`       | 字段匹配     |
| 记录简写 | `case (:x) {}`           | 捕获同名字段 |
| 逻辑组合 | `case a and b {}`        | 两者都满足   |
| 守卫     | `case x if x > 5 {}`     | 附加条件     |

---

## 常用内建函数

### 集合操作

| 函数                                                        | 说明                                      |
| ----------------------------------------------------------- | ----------------------------------------- |
| `len(arr)`                                                  | 数组或记录的长度                          |
| `keys(record)`                                              | 记录的键列表                              |
| `values(record)`                                            | 记录的值列表                              |
| `map(data, fn)`                                             | 映射每个元素                              |
| `filter(data, fn)`                                          | 过滤元素                                  |
| `filter_map(data, fn)`                                      | 映射并过滤 nil                            |
| `find(data, fn)`                                            | 查找第一个匹配元素（返回 `(key, value)`） |
| `all(data, fn)`                                             | 所有元素满足条件                          |
| `any(data, fn)`                                             | 存在元素满足条件                          |
| `sort(data, comparer?)` / `sort_by(data, keyFn, comparer?)` | 排序                                      |
| `unique(data, equal?)` / `unique_by(data, keyFn, equal?)`   | 去重                                      |
| `join(arr, sep)`                                            | 数组连接为字符串                          |
| `with(data, ..entries)`                                     | 创建更新后的副本                          |

### 字符串操作

| 函数                       | 说明           |
| -------------------------- | -------------- |
| `split(str, sep)`          | 按分隔符分割   |
| `trim(str)`                | 去除首尾空白   |
| `replace(str, from, to)`   | 替换子字符串   |
| `starts_with(str, prefix)` | 是否以前缀开头 |
| `ends_with(str, suffix)`   | 是否以后缀结尾 |

### 类型与转换

| 函数                     | 说明           |
| ------------------------ | -------------- |
| `type(value)`            | 返回类型字符串 |
| `to_number(v, default?)` | 转为数字       |
| `to_string(v)`           | 转为字符串     |

❌ 不支持 `to_boolean(v)` 及隐式转换为布尔值，依据需求生成合适的布尔表达式。

### 数学

| 函数                                | 说明     |
| ----------------------------------- | -------- |
| `abs(x)`                            | 绝对值   |
| `sqrt(x)`                           | 平方根   |
| `floor(x)` / `ceil(x)` / `round(x)` | 取整     |
| `min(..args)` / `max(..args)`       | 最值     |
| `sum(..args)`                       | 求和     |
| `PI` / `E`                          | 数学常量 |

---

## 编码规范总结（Rust Clippy 风格）

1. **隐式返回优先**：函数末尾用表达式返回，不写 `return`；仅提前退出时使用 `return`
2. **最小可变性**：优先 `let`，仅需重新赋值时才 `let mut`
3. **链式管道**：多步数据转换使用 `::` 链，避免深层嵌套
4. **模式匹配**：用 `match` 替代复杂的 `if/else if` 链
5. **空安全**：用 `??` 提供默认值，避免随意使用 `!`
6. **字符串插值**：用 `$name`、`$(expr)`，不要用 `+` 拼接字符串
7. **函数粒度**：将重复逻辑抽取为具名函数，匿名函数仅用于简短的内联逻辑

---

## 常见错误

```mira
// ❌ 使用 + 拼接字符串
"Hello, " + name

// ✅ 使用插值
"Hello, $name"

// ❌ 修改不可变变量
let arr = [1, 2, 3];
arr = [..arr, 4];   // 错误！

// ✅ 使用 let mut 或构造新绑定
let arr2 = [..arr, 4];

// ⚠️ 在末尾使用 return
fn f(x) { return x * 2; }

// ✅ 隐式返回
fn f(x) { x * 2 }

// ⚠️ 不必要的 let mut
let mut x = 10;   // 若后续从未重新赋值
// ✅
let x = 10;
```

---

## 代码验证工作流

详细用法参见 **mirascript-cli** 技能。核心流程如下：

### 步骤 1：用 `run -` 验证代码片段

```sh
npx @mirascript/cli - <<EOF
<生成的代码>
EOF
```

退出码为 `0` 且输出符合预期时，验证通过。

### 步骤 2：根据错误输出修复

CLI 错误会显示错误类型、行列号和指示符，根据提示定位并修复问题后**重新运行验证**，直到通过。

### 步骤 3：验证完成后可选格式化

```sh
npx @mirascript/cli format -w script.mira
```

### 示例：验证一个函数

```sh
npx @mirascript/cli - <<EOF
fn fib(n) {
  if n <= 1 { return n; }
  fib(n - 1) + fib(n - 2)
}
fib(10)
EOF
# 预期输出：55
```

若输出与预期不符或退出码非 `0`，必须修正代码并重新验证。
