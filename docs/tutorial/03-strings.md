# 字符串

字符串用于表示文本数据。MiraScript 提供了灵活的字符串语法和强大的插值功能。

## 字符串字面量

MiraScript 支持三种引号来创建字符串：

```mira
let s1 = 'hello';     // 单引号
let s2 = "world";     // 双引号
let s3 = `多行
字符串`;              // 反引号

debug_print(s1);
debug_print(s2);
debug_print(s3);
```

三种引号的功能完全相同，区别在于，使用某种引号时，字符串内部可以直接包含其他两种引号，而不需要转义。

```mira
debug_print("It's a test");       // 双引号中直接用单引号
debug_print('She said "hi"');     // 单引号中直接用双引号
```

## 转义字符

使用反斜杠 `\` 可以在字符串中插入特殊字符：

| 转义字符 | 含义                                     |  转义字符  | 含义                                                 |
| :------: | ---------------------------------------- | :--------: | ---------------------------------------------------- |
|   `\'`   | 单引号                                   |    `\t`    | 制表符                                               |
|   `\"`   | 双引号                                   |    `\b`    | 退格符                                               |
| `` \` `` | 反引号                                   |    `\f`    | 换页符                                               |
|   `\\`   | 反斜杠                                   |    `\v`    | 垂直制表符                                           |
|   `\n`   | 换行符                                   |    `\0`    | 空字符                                               |
|   `\r`   | 回车符                                   |    `\$`    | 美元符号                                             |
|  `\xXX`  | ASCII 字符，`XX` 是小于 128 的十六进制数 | `\u{XXXX}` | Unicode 字符，`XXXX` 是合法 Unicode 码点的十六进制数 |

```mira
debug_print("第一行\n第二行");
debug_print("名字\t年龄");
debug_print("价格：\$99");
```

## 字符串插值

字符串插值是 MiraScript 最实用的特性之一，使用 `$` 将变量或表达式的值嵌入字符串中：

### 插入变量

使用 `$变量名` 直接插入变量的值：

```mira
let name = "小明";
let age = 20;
debug_print("我叫 $name，今年 $age 岁");
```

### 插入表达式

使用 `$(..)` 插入任意表达式的计算结果：

```mira
let a = 3;
let b = 4;
debug_print("$a + $b = $(a + b)");
debug_print("$a × $b = $(a * b)");
debug_print("√($a² + $b²) = $(sqrt(a^2 + b^2))");
```

### 插入块表达式

使用 `${..}` 插入一个代码块的结果：

```mira
let score = 85;
debug_print("成绩：$score，等级：${ if score >= 90 { "优" } else if score >= 60 { "及格" } else { "不及格" } }");
```

### 格式化输出

在插值表达式 `$(..)` 后添加 `:格式` 可以控制输出格式：

```mira
let pi = PI;
debug_print("π ≈ $(pi:.4)");       // 保留 4 位小数
debug_print("π ≈ $(pi:.2)");       // 保留 2 位小数
```

## 逐字字符串

使用 `@"..."@` 创建逐字字符串，其中除 `$` 外的所有字符都被视为普通字符，不会被转义：

```mira
let path = @"C:\Users\name\file.txt"@;
debug_print(path);  // 输出：C:\Users\name\file.txt

// 普通字符串中需要转义反斜杠
let path2 = "C:\\Users\\name\\file.txt";
debug_print(path2); // 输出相同
```

逐字字符串中仍然可以使用 `$` 进行插值：

```mira
let name = "Alice";
debug_print(@"Hello, $name! Path: C:\Users"@);
```

要在逐字字符串中使用 `$` 或 `"@`，使用更多的 `@` 包裹：

```mira
// 使用与 @ 数量相同的 $ 来插值
let name = "Bob";
debug_print(@@"Hello, $$name! This is a literal @""@ and $ symbol."@@);
```

## 常用字符串函数

### 查找与判断

```mira
let sentence = "Hello, MiraScript!";
debug_print("包含 'Mira':", contains(sentence, "Mira"));           // true
debug_print("以 'Hello' 开头:", starts_with(sentence, "Hello"));   // true
debug_print("以 '!' 结尾:", ends_with(sentence, "!"));             // true
```

### 拆分与连接

```mira
// 拆分字符串为数组
let csv = "apple,banana,cherry";
let fruits = split(csv, ",");
debug_print("拆分结果:", fruits);

// 连接数组为字符串
let joined = join(fruits, " | ");
debug_print("连接结果:", joined);
```

### 去除空白与替换

```mira
let messy = "  hello, world  ";
debug_print("去除两端空白:", trim(messy));
debug_print("去除开头空白:", trim_start(messy));
debug_print("去除结尾空白:", trim_end(messy));

let text = "Hello, World!";
debug_print("替换:", replace(text, "World", "MiraScript"));
```

### 转为字符数组

```mira
let word = "Hi!";
let characters = chars(word);
debug_print("字符数组:", characters);  // ["H", "i", "!"]
```

:::tip
使用 `len(chars(str))` 获取字符串长度。
:::

## 类型转换

使用 [`to_string()`](../lib/00-global.md#fn-to_stringdata-fallback) 可以将其他类型转为字符串：

```mira
debug_print(to_string(42));      // "42"
debug_print(to_string(true));    // "true"
debug_print(to_string(nil));     // ""（空字符串）
debug_print(to_string([1,2,3])); // "1, 2, 3"
```

## 小结

- 三种引号 `'`、`"`、`` ` `` 可互换使用，字符串中支持换行
- `$name`、`$(expr)`、`${block}` 进行字符串插值
- `@"..."@` 逐字字符串用于包含大量反斜杠的文本
- 内置 `len`、`contains`、`split`、`join`、`trim`、`replace` 等字符串处理函数
