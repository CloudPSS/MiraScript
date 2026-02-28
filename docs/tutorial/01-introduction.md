# 初识 MiraScript

MiraScript 是一门**表达式优先**的脚本语言，设计用于嵌入到其他应用程序中。它的核心特点包括：

- **表达式优先**：几乎所有语法结构都是表达式，都有返回值
- **不可变数据**：数据一旦创建就不能被修改，更安全也更易于理解
- **空安全**：访问不存在的属性不会报错，而是返回 [`nil`](../references/keyword/nil.md)

本教程将从最简单的表达式开始，带你一步步掌握 MiraScript 的核心功能。

:::tip
本教程中的代码块可以直接编辑和运行，建议你动手修改代码，观察运行结果。
:::

## 第一个表达式

MiraScript 是一门表达式语言，你可以直接书写数学表达式：

```mira
1 + 2
```

表达式的值就是计算结果，这里是 `3`。

## 输出结果

使用 [`debug_print` 函数](../lib/00-global.md#fn-debug_printargs) 可以将值输出到控制台，方便我们查看计算结果：

```mira
debug_print("Hello, MiraScript!");
debug_print(1 + 2);
debug_print("1 + 2 =", 1 + 2);
```

`debug_print` 可以接受多个参数，参数之间用逗号分隔。

## 语句与分号

MiraScript 中的语句以分号 `;` 结尾。一个程序可以包含多条语句：

```mira
debug_print("第一行");
debug_print("第二行");
debug_print("计算结果:", 10 * 2 + 3);
```

## 注释

注释是写给阅读代码的人看的说明文字，不会被执行。MiraScript 支持两种注释：

```mira
// 这是单行注释，从 // 开始直到行末

/* 这是多行注释
   可以跨越多行
   适合书写较长的说明 */

debug_print("注释不影响代码执行"); // 行尾也可以写注释
```

## 基本数据类型概览

MiraScript 有以下几种基本数据类型：

```mira
// 数字（number）—— 64 位浮点数
debug_print(42);
debug_print(3.14);

// 字符串（string）—— 文本
debug_print("你好，世界");

// 布尔（boolean）—— 真或假
debug_print(true);
debug_print(false);

// 空值（nil）—— 表示"没有值"
debug_print(nil);
```

使用 [`type()` 关键字](../references/keyword/type.md) 可以查看一个值的类型：

```mira
debug_print(type(42));        // "number"
debug_print(type("hello"));   // "string"
debug_print(type(true));      // "boolean"
debug_print(type(nil));       // "nil"
```

## 小结

你已经了解了 MiraScript 的基本概念：

- 使用 `debug_print()` 输出值
- 书写表达式和语句
- 使用 `//` 和 `/* */` 添加注释
- MiraScript 有数字、字符串、布尔、空值四种基本数据类型

接下来，我们将深入学习数值与运算。
