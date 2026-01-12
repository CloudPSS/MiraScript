---
mdx:
  format: md
---

# MiraScript 速查表

MiraScript 是一门表达式优先、不可变数据为核心的脚本语言。

## 基础

- **注释**：`//` 单行；`/* ... */` 多行（不可嵌套）。
- **标识符**：以 `$`/`@`/字母/`_` 开头，可包含数字和 `_`；`@` 开头必须用 `const`，`$` 和 `__` 前缀保留。
- **字面量**：`nil`、布尔（`true` / `false`）、数字（含二 `0b101` / 八 `0o777` /十六进制 `0xcf`、`inf` / `nan`、下划线分隔）、字符串（单 `'` / 双 `"` / 反引号 `` ` ``，支持转义与逐字字符串 `@"..."@`）。
- **记录**：`(key: value, ..spread)`；可省略键名或使用序数；`?:` 省略为 `nil` 的键。
- **数组**：`[1, 2, ..spread]`，支持范围填充 `[start..end]` / `[start..<end]`。

## 值与可变性

- 除 `function`、`module`、`extern`，其余类型皆为值语义且不可变。
- 访问不存在的成员或对 `nil` 连续取属性默认返回 `nil`，使用 `!` 断言非空。
- 空合并 `??` 提供默认值；逻辑与或 `&&` / `||`、空合并均短路。

## 表达式优先级（部分）

1. 后缀：`.`, `[]`, 切片、调用、`::`, `!`
2. 幂：`^`
3. 前缀：`!`, `+`, `-`
4. 乘除余：`*`, `/`, `%`
5. 加减：`+`, `-`
6. 模式匹配：`is`
7. 关系、相等：`in`, `>`, `>=`, `<`, `<=`, `==`, `!=`, 近似等于 `=~`, 不近似等于 `!~`
8. 逻辑：`&&`, `||`, `??`

## 语句与绑定

- `const @NAME = expr;` 定义常量。
- `let pattern = expr;` 使用模式解构绑定；`mut` 声明可变变量。
- 赋值与数学运算支持 `+=`, `-=`, `*=` 等写法。

## 控制流

- `if condition { ... } else { ... }`，条件必须为布尔。
- `match expr { case pattern [if guard] { ... } ... }`，未匹配返回 `nil`。
- 循环：`for pattern in iterable { ... } [else ...]`、`while cond { ... } [else ...]`、`loop { ... }`；`break value;` 返回循环值，`continue;` 跳过本次。

## 函数与调用

- 声明：`fn fn_name(a, b) { body }`；单参数可省括号 `fn sq { it^2 }` ，参数名为 `it`。
- 调用：`fn_name(args)`；使用 `..array` 展开参数。
- 扩展调用：`value::func(other)` 相当于 `func(value, other)`，便于链式组合。

## 模式匹配速览

- **字面量/常量**：`case 1 { ... }`、`case @x { ... }`。
- **关系/范围**：`case > 0 { ... }`、`case =~ 0.5 { ... }`、`case 1..10 { ... }`、`case 0..<1 { ... }`。
- **变量绑定/弃元**：`case mut x { ... }`、`case y { ... }`、`case _ { ... }`。
- **记录/数组**：`case (:x, y: 0, ..rest) { ... }`、`case [head, ..tail] { ... }`。
- **逻辑**：`pattern and pattern`、`pattern or pattern`、`not pattern`（不短路）。

## 常用内建与技巧

- `type(value)` 返回值类型字符串。
- 字符串插值：`"hello, $name"`，表达式形式 `"$(expr)"` `"${exprs}"`；逐字字符串用匹配数量的 `$`。
- 访问数组切片：`arr[1..3]`、`arr[..<-1]`；负索引从尾部开始。

## 推荐用法

- 使用链式 `::` 编排数据流，提升可读性。
- 利用 `match` + 模式组合可读的业务分支。
- 避免依赖隐式类型转换，必要时显式调用转换函数 `to_*`。

## MiraScript vs MathJS

|              | MiraScript                                  | MathJS                                                      |
| ------------ | ------------------------------------------- | ----------------------------------------------------------- |
| 变量声明     | `let mut x = 1;` / `let x = 1;`             | `x = 1;`                                                    |
| 数组/矩阵    | `[[1 ,2 ,3], [4, 5, 6]]`                    | `[1, 2, 3; 4, 5, 6]`                                        |
| 序列         | `[1..9]` / `[1..<10]`                       | `1:9`                                                       |
| 记录（对象） | `(a: 1, b: 'str')` / `{"a": 1, "b": "str"}` | `{a: 1, b: 'str'}` / `{"a": 1, "b": "str"}`                 |
| 格式化字符串 | `'$(data.value) kW'`                        | `print('$data kW', { data: data.value })`                   |
| 矩阵操作     | `matrix.add(mA, mB)`                        | `mA + mB`                                                   |
| 函数调用     | `map(values(cells), fn { it.key })`         | `mapper(v, i, arr) = v.key; values(cells).map(mapper)`      |
| 数值比较     | `value !~ 1` / `x =~ y`                     | `value != 1` / `x == y`                                     |
| 字符串比较   | `str == "Alice"`                            | `equalText(str, "Alice")`                                   |
| 函数声明     | `fn add(x, y) { x + y }`                    | `add(x, y) = x + y`                                         |
| 类型判断     | `type(x) == 'string'`                       | `is(x, 'string')`                                           |
| 空值合并     | `context.Vm ?? 1`                           | `is(context, 'Object') ? (context.Vm ? context.Vm : 1) : 1` |

<style>
  body > hr:first-child, body > #mdxformat-md {
    display: none;
  }
  #mirascript-vs-mathjs + table {
    th:first-child {
      width: 15%;
    }
    th:nth-child(2) {
      width: 45%;
    }
  }

  @media print {
    code {
      font-family: consolas;
      font-size: 1em;
      line-height: 1.357em;
      background-color: #8883;
      padding: 1px 3px;
      border-radius: 4px;
      print-color-adjust: exact;
    }
  }
</style>
