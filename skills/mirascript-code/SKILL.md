---
name: mirascript-code
description: 编写、修改、解释和审查正确、惯用的 MiraScript 代码。处理 .mira 或 .miratpl 文件、MiraScript 代码片段、语言语法与内建 API、将其他语言逻辑改写为 MiraScript，或诊断语言级错误时使用；可执行时配合 mirascript-cli 完成运行和格式化验证。
---

# 编写 MiraScript

## 工作流

1. 先确认目标是脚本、模板、嵌入式片段还是宿主提供的运行环境，并保留宿主约定的全局变量、模块和入口函数。
2. 涉及具体语法、模式、模板或内建函数时，读取 [references/language-guide.md](references/language-guide.md)。只读取与任务相关的章节。
3. 在 MiraScript 仓库内工作时，以现有 `.mira` 示例、`docs/tutorial/`、`docs/references/` 和运行时导出的库函数为当前事实来源；不要凭相似语言猜测语法或 API。
4. 先写最小可运行实现，再用 `$mirascript-cli` 执行并格式化。每次修复后重新运行；同时检查退出码、标准错误和实际输出。
5. 无法执行时，明确说明未验证的边界，不要把静态检查描述成运行通过。

## 必守语义

- 将 MiraScript 视为表达式优先语言。使用块或函数的最后一个表达式作为结果，不要在它后面加分号。
- 用分号结束 `let`、赋值、提前 `return`/`break`/`continue`，以及仅为副作用执行的调用。
- 默认使用 `let`；仅在绑定后续确实会被重新赋值时使用 `let mut`。
- 把数组和记录视为不可变数据。使用展开语法或 `with` 创建新值，不要写元素或属性赋值。
- 用 `$name`、`$(expression)` 或 `${ block }` 拼接字符串；不要用 `+` 连接字符串。
- 让 `if`、`while` 和过滤谓词返回布尔值；不要依赖其他语言的真假值规则。
- 用 `??` 提供 `nil` 默认值；仅在能够证明非空时使用 `!`。
- 只调用已确认存在的内建函数或宿主注入函数。MiraScript 没有通用的 `to_boolean` 内建函数。

## 惯用写法

优先写隐式返回，并把 `return` 留给提前退出：

```mira
fn classify(n) {
  if n < 0 { return "negative"; }
  if n == 0 { "zero" } else { "positive" }
}
```

用扩展调用 `::` 表达从左到右的数据流；`value::f(arg)` 等价于 `f(value, arg)`：

```mira
let result = values
  ::filter(fn { it > 0 })
  ::map(fn { it * 2 })
  ::sort();
```

用展开构造不可变更新：

```mira
let appended = [..items, next];
let renamed = (..user, name: "Mira");
let nested = config::with(["server", "port"], 8080);
```

在分支取决于值的结构、范围或守卫时使用 `match`；简单的二选一仍使用 `if`。

## 交付检查

- 确认变量可变性、分号和最后表达式符合预期。
- 确认范围 `..` 与半开范围 `..<` 的上界语义正确。
- 确认数组/记录更新没有写成原地修改。
- 确认字符串插值和 shell 转义没有混淆。
- 确认使用了真实存在且参数顺序正确的内建函数。
- 对有代表性的正常、边界和 `nil` 输入执行代码；需要格式统一时再格式化目标文件。
