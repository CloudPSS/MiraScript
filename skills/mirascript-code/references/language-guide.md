# MiraScript 语言指南

按任务读取相关章节；不要为了简单改动加载全部内容。在 MiraScript 仓库内工作时，若这里与当前源码、测试或文档冲突，以仓库当前实现为准。

## 目录

- [程序与数据模型](#程序与数据模型)
- [函数、控制流与循环](#函数控制流与循环)
- [数组、记录与不可变更新](#数组记录与不可变更新)
- [字符串与模板](#字符串与模板)
- [模式与解构](#模式与解构)
- [扩展调用与空安全](#扩展调用与空安全)
- [模块](#模块)
- [常用内建函数](#常用内建函数)
- [常见错误](#常见错误)

## 程序与数据模型

MiraScript 是动态类型、表达式优先的嵌入式语言。常见值包括 `number`、`string`、`boolean`、`nil`、`array`、`record`、`function` 和宿主提供的 `extern`。

```mira
let immutable = 1;
let mut counter = 0;
counter += 1;
const @MAX_SIZE = 100;

// 块的最后表达式是块结果
let label = {
  let doubled = counter * 2;
  "value=$doubled"
};
```

使用 `type(value)` 查看类型。条件位置使用布尔表达式，不要假设数字、字符串或集合可隐式充当布尔条件。

## 函数、控制流与循环

```mira
fn add(x, y) { x + y }
fn double { it * 2 }

let multiply = fn (x, y) { x * y };
let square = fn { it^2 };

fn sum_all(first, ..rest) {
  first + sum(..rest)
}
```

对正常结果使用最后表达式；仅对提前退出使用 `return value;`。`if`、`match` 和循环都能产生值。

```mira
let label = if score >= 60 { "pass" } else { "fail" };

let category = match value {
  case nil { "missing" }
  case 0 { "zero" }
  case > 0 and < 10 { "small positive" }
  case _ { "other" }
};

for item in items { debug_print(item); }
while counter > 0 { counter -= 1; }

let found = loop {
  if ready { break result; }
};
```

`for` 可遍历数组和记录；遍历记录时循环变量是键。使用 `continue;` 跳过本轮。需要正常结束时的替代结果，可使用循环的 `else` 分支。

## 数组、记录与不可变更新

```mira
let inclusive = [1..5];       // [1, 2, 3, 4, 5]
let exclusive = [1..<5];      // [1, 2, 3, 4]
let merged = [..left, ..right];

let first = merged[0];
let last = merged[-1];
let closed_slice = merged[1..3];
let open_slice = merged[1..<3];

let person = (name: "Mira", age: 20);
let point = (10, 20);
let compact = (:name, :age);

let next_items = items::with(0, replacement);
let next_person = (..person, age: 21);
let next_config = config::with(["server", "port"], 8080);
```

访问缺失索引或属性会得到 `nil`。不要写 `items[0] = value` 或 `person.name = value`；即使绑定用 `let mut` 声明，数据本身仍然不可变，只能把新数据重新赋给绑定。

单个未命名记录元素必须写尾逗号：`(value,)`；`(value)` 只是分组表达式。记录也可使用 JSON 风格的 `{ "key": value }` 语法。

## 字符串与模板

单引号、双引号和反引号都可创建字符串。使用插值而非 `+`：

```mira
let name = "Mira";
let age = 20;

let greeting = "Hello, $name";
let next_age = "next age=$(age + 1)";
let state = "state=${ if age >= 18 { "adult" } else { "minor" } }";
(greeting, next_age, state)
```

使用 `@"..."@` 创建逐字字符串。增加两端 `@` 的数量以容纳 `"@`，并使用相同数量的 `$` 触发插值：

```mira
let path = @"C:\Users\name"@;
let text = @@"Hello, $$name! This is a literal @""@ and $ symbol."@@;
```

`.miratpl` 文件把普通文本作为模板内容，并使用相同的 `$name`、`$(expression)` 和 `${ block }` 插入值。通过 stdin 或 `--eval` 执行模板时，显式启用 CLI 的 `--template`。

## 模式与解构

```mira
match value {
  case 42 { "literal" }
  case 1..10 { "inclusive range" }
  case [head, ..tail] { "array" }
  case (name: n, age: a) { "$n: $a" }
  case x if x is > 10 { "guarded $x" }
  case _ { "fallback" }
}
```

用 `and`、`or`、`not` 组合模式；用 `if` 添加守卫。`let` 也接受数组和记录模式：

```mira
let [first, ..rest] = values;
let (:name, age: years, ..) = person;
```

## 扩展调用与空安全

`value::fn(args)` 把 `value` 作为 `fn` 的第一个参数。优先用它表达可读的数据处理链，但不要为了链式风格打乱清晰的参数顺序。

```mira
let result = data
  ::filter(fn { it.active })
  ::sort_by(fn { it.score })
  ::map(fn { it.name });

let port = config.server.port ?? 8080;
let required = maybe_value!;
```

属性链默认空安全：中间值为 `nil` 或属性不存在时，后续访问继续得到 `nil`。`!` 在值为 `nil` 时抛错。

## 模块

使用 `mod` 组织成员，并只用 `pub` 暴露外部需要访问的部分：

```mira
mod math_utils {
  pub fn square { it^2 }
  pub const @FACTOR = 2;

  fn helper { it * @FACTOR }
  pub fn doubled_square { helper(it)::square() }
}

math_utils.square(5)
```

模块可嵌套。模块内的可变成员只能通过模块内部逻辑重新赋值，不要从外部直接给模块属性赋值。

## 常用内建函数

先确认宿主没有替换或限制全局库，再使用下列常见函数。

| 类别   | 函数                                                                                                                                                                           |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 集合   | `len`, `keys`, `values`, `entries`, `map`, `filter`, `filter_map`, `find`, `fold`, `all`, `any`, `sort`, `sort_by`, `unique`, `unique_by`, `reverse`, `flatten`, `zip`, `with` |
| 字符串 | `chars`, `split`, `join`, `trim`, `trim_start`, `trim_end`, `replace`, `contains`, `starts_with`, `ends_with`, `to_uppercase`, `to_lowercase`                                  |
| 转换   | `type`, `to_number`, `to_string`, `to_json`, `from_json`                                                                                                                       |
| 数学   | `abs`, `sqrt`, `floor`, `ceil`, `round`, `min`, `max`, `sum`, `product`, `PI`, `E`                                                                                             |
| 调试   | `debug_print`, `panic`                                                                                                                                                         |

回调通常接收 `(value, key, input)`，短回调可只使用隐式参数 `it`。`find` 返回首个匹配项的 `(key, value)`，找不到时返回 `nil`。`with(data, key, value, ...)` 的键值参数必须成对出现；数组键表示嵌套路径。

## 常见错误

- 不要写 `"Hello, " + name`；改为 `"Hello, $name"`。
- 不要给 `let` 绑定重新赋值；确有需要时声明 `let mut`。
- 不要原地修改数组或记录；创建副本并在需要时重新绑定。
- 不要在函数最后写多余的 `return value;`；保留最后表达式即可。
- 不要把 `..<` 误写为其他语言的 `...`，也不要混淆闭区间 `..`。
- 不要发明类似语言中的 API；先查当前文档、运行时导出或 CLI 错误。
