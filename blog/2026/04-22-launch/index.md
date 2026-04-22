---
draft: true
---

# MiraScript：表达式优先的脚本语言

在能源电力建模仿真、数字孪生场景中，复杂逻辑表达、算例配置、数据处理与批量自动化的需求持续增长。此前在 XStudio 中广泛使用的 Math.js 擅长纯数学计算，但缺少**作用域、控制流、模式匹配、空安全**等现代脚本能力，难以支撑规模化、工程化的脚本开发。

为此，CloudPSS 团队自研 **MiraScript** ——一门以**表达式优先、不可变数据为核心**的动态类型脚本语言，专为嵌入仿真平台、数字孪生工坊与自动化工具链设计，语法简洁接近 Rust，安全、高效、易嵌入、易迁移，彻底解决复杂场景下的脚本编写与维护难题。

<!-- truncate -->

## 设计哲学

MiraScript 围绕四大核心原则设计，兼顾表达力与工程健壮性：

1. **表达式优先**：几乎所有语法结构均为表达式，可直接赋值、返回、组合，代码更精简。

2. **不可变默认**：数据默认不可变，`mut` 显式声明可变，消除意外副作用，更适合表达式计算。

3. **空安全**：原生支持 `nil` 安全访问、空合并 `??`、非空断言 `!`，大幅减少崩溃与边界判断。

4. **嵌入优先**：轻量编译器、多 runtime 适配、极简集成成本，深度适配 CloudPSS 全栈环境。

## 核心语言特性

### 表达式优先：一切皆可返回

`if`/`match`/`loop` 均为表达式，可直接绑定变量，告别冗余赋值。

```mira
let score = 90;
let status = if score > 60 { "pass" } else { "fail" };
let level = match score {
  case 90..100 { "优秀" }
  case 60..<90 { "及格" }
  case _ { "不及格" }
};
debug_print(score, status, level);
```

### 不可变数据 + 显式可变

默认不可变，需要修改时用 `mut`，逻辑清晰、更安全。

```mira
const @PI = 3.14159;       // 常量
let data = [1, 2, 3];    // 不可变
let mut count = 0;       // 可变
count += 1;
```

### 空安全：告别空指针崩溃

访问不存在成员返回 `nil`，`??` 提供默认值，`!` 断言非空。

```mira
let user = (name: 'Alice');
let city = user.address.city ?? "未知"; // 空合并
debug_print(user.name!, city);          // 非空断言
```

### 强大模式匹配：替代冗长 `if else`

支持字面量、范围、解构、守卫、逻辑组合，分支逻辑一目了然。

```mira
let item = 12;
match item {
  case > 0 and < 100 { "有效范围" }
  case [h, ..t] { "数组：头部=${h}" }
  case (:id) { "记录：ID=${id}" }
}
```

### 扩展调用 `::`：数据流更清晰

`value::func(arg)` ≡ `func(value, arg)`，天然支持链式处理数据。

```mira
let result = [5, 2, 8, 1]
  ::map(fn { it * 2 })
  ::filter(fn { it > 3 })
  ::sort();
```

### 原生字符串插值

支持变量与表达式插值，书写直观。

```mira
let name = 'MiraScript';
let arr = [1, 2, 3];
debug_print("Hello, $name!");
debug_print("长度：$(arr::len())");
```

### 简洁函数语法

单参数可省略括号，参数默认名 `it`，快速编写小函数。

```mira
fn double { it * 2 }
fn add(a, b) { a + b }
```

## 技术架构

MiraScript 采用**编译 + 跨平台执行**架构，性能与兼容性兼备：

1.  **Rust 编译器**

    源码解析 → 生成字节码，执行效率高、安全性强。

2.  **双 Runtime 执行环境**

    提供 **JavaScript / Python** 字节码转译能力，**一次编写，前后端一致运行**，无缝对接 CloudPSS 前端与 SDK。

3.  **专业编辑工具链**

    支持 [Monaco](https://www.npmjs.com/package/@mirascript/monaco) / [VS Code](https://marketplace.visualstudio.com/items?itemName=CloudPSS.mirascript) 插件，提供：
    - 语法高亮、自动补全、签名提示
    - 实时代码诊断、重构、格式化
    - 内嵌提示（Inlay Hints）

## 开放开源

MiraScript 坚持开放理念，面向全球开发者开放：

- 官方文档：[https://mira.cloudpss.net/](https://mira.cloudpss.net/)
- 开源仓库：[https://github.com/CloudPSS/MiraScript](https://github.com/CloudPSS/MiraScript)
- 欢迎提交 Issue、PR、参与讨论，共同完善 MiraScript 脚本语言。

## 结语

MiraScript 不止是一门脚本语言，更是 CloudPSS 为能源电力仿真、数字孪生场景打造的**工程化、现代化、安全高效**的逻辑表达基础设施。它以简洁语法承载复杂逻辑，以开放架构融入生态，让仿真自动化、配置可编程、处理规模化更简单。

未来，我们将持续迭代语法、扩展仿真专属 API、深化全产品栈融合，与开发者、用户一同构建更强大的仿真脚本生态。
