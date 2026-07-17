# 布尔与条件

布尔类型用于表示逻辑上的"真"和"假"，是控制程序流程的基础。

## 布尔值

MiraScript 中的布尔类型只有两个值：[`true`](../references/keyword/true.md)（真）和 [`false`](../references/keyword/false.md)（假）。

```mira
let is_student = true;
let has_graduated = false;
debug_print("是学生:", is_student);
debug_print("已毕业:", has_graduated);
```

## 比较运算符

比较运算符将两个值进行比较，返回布尔值：

```mira
debug_print("3 > 2:", 3 > 2);     // true
debug_print("3 < 2:", 3 < 2);     // false
debug_print("3 >= 3:", 3 >= 3);   // true
debug_print("3 <= 2:", 3 <= 2);   // false
debug_print("3 == 3:", 3 == 3);   // true
debug_print("3 != 4:", 3 != 4);   // true
```

### 严格相等

[`==` 运算符](../references/operator/equal.md) 是**严格相等**，不会进行类型转换。不同类型的值永远不相等：

```mira
debug_print(1 == "1");     // false，number 和 string 不同类型
debug_print(0 == false);   // false，number 和 boolean 不同类型
```

### 近似相等

[`=~` 运算符](../references/operator/tilde_equal.md) 用于近似比较，对数字允许微小误差，对字符串忽略大小写：

```mira
debug_print(0.1 + 0.2 == 0.3);   // false，浮点精度问题
debug_print(0.1 + 0.2 =~ 0.3);   // true，近似相等
debug_print("Hello" =~ "hello"); // true，忽略大小写
```

### 隐式类型转换

在进行**比较**运算和**近似相等**运算时，MiraScript 会对两侧的操作数进行类型转换：

- 至少有一个操作数是 `number` 时，另一个操作数会被转换为 `number`；
- 否则，至少有一个操作数是 `string` 时，另一个操作数会被转换为 `string`；
- 否则，两个操作数都会被转换为 `number`。

```mira
debug_print(1 == "1");       // false，严格相等不转换类型
debug_print(1 =~ "1");       // true，近似相等会转换 "1" 为数字 1
debug_print(0 =~ false);     // true，近似相等会转换 false 为数字 0
debug_print("0" =~ false);   // false，近似相等会转换 false 为字符串 "false"，与 "0" 不相等
debug_print(false =~ false); // true，两侧均非 number/string，转换为数字 0，相等
```

:::tip

为了避免预期外的隐式转换，建议在比较时确保其中一个操作数为 `number` 类型（此时另一个操作数的转换行为固定），或者明确进行类型转换。

:::

## 逻辑运算符

逻辑运算符用于组合布尔表达式：

```mira
let a = true;
let b = false;

// 逻辑与：两者都为 true 才是 true
debug_print("true && false:", a && b);   // false

// 逻辑或：至少一个为 true 就是 true
debug_print("true || false:", a || b);   // true

// 逻辑非：取反
debug_print("!true:", !a);               // false
debug_print("!false:", !b);              // true
```

也可以使用关键字 `and`、`or`、`not` 替代符号：

```mira
debug_print(true and false);  // false
debug_print(true or false);   // true
debug_print(not true);        // false
```

### 短路求值

逻辑运算符支持**短路求值**：

- `&&` 左侧为 `false` 时，右侧不会被计算
- `||` 左侧为 `true` 时，右侧不会被计算

```mira
let x = false;
let y = true;
let result = x && y; // y 不会被求值，结果为 false
debug_print(result);
```

## `if` 表达式

`if` 用于根据条件执行不同的代码。在 MiraScript 中，`if` 是一个**表达式**，也就是说它有返回值：

```mira
let score = 85;

if score >= 60 {
  debug_print("及格了！")
}
```

### `if/else`

```mira
let temperature = 35;

let advice = if temperature > 30 {
  "天气很热，注意防暑"
} else {
  "天气不错"
};

debug_print(advice);
```

### `if/else if/else`

```mira
let score = 78;

let grade = if score >= 90 {
  "优秀"
} else if score >= 80 {
  "良好"
} else if score >= 60 {
  "及格"
} else {
  "不及格"
};

debug_print("成绩 $score 分，评级：$grade");
```

:::warning[条件必须是布尔值]

MiraScript **不支持**将数字、字符串等隐式转换为布尔值。以下写法会产生运行时错误：

```mira
if 0 { }          // 错误！0 不是布尔值
if "" { }         // 错误！空字符串不是布尔值
if nil { }        // 错误！nil 不是布尔值
```

正确做法是使用比较或 `is` 运算符：

```mira
let x = 0;
if x == 0 { debug_print("x 是零") }
if x is nil { debug_print("x 是 nil") }
```

:::

## 条件运算符

条件运算符 `? :` 是 `if/else` 的简写，适合简单的二选一：

```mira
let age = 20;
let status = age >= 18 ? "成年" : "未成年";
debug_print(status);
```

## 实际应用

```mira
// 判断一个年份是否是闰年
let year = 2024;
let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
debug_print("$year 年是闰年吗？", is_leap);

// BMI 计算与评估
let weight = 70;   // 体重（kg）
let height = 1.75; // 身高（m）
let bmi = weight / height^2;

let category = if bmi < 18.5 {
  "偏瘦"
} else if bmi < 24 {
  "正常"
} else if bmi < 28 {
  "偏胖"
} else {
  "肥胖"
};

debug_print("BMI = $(bmi:.1)，体型：$category");
```

## 小结

- 布尔值只有 `true` 和 `false`
- `==` 严格相等（不转换类型），`=~` 近似相等
- `&&`/`||`/`!` 或 `and`/`or`/`not` 进行逻辑运算
- `if/else` 是**表达式**，有返回值
- 条件必须是 `boolean` 类型，不支持隐式转换
