# 模式匹配

模式匹配是 MiraScript 最强大的特性之一，可以根据数据的结构和内容进行分支判断，同时提取其中的数据。

## `match` 表达式基础

`match` 表达式对一个值依次尝试各个 `case` 分支，执行第一个匹配成功的分支：

```mira
let x = 2;
let result = match x {
  case 1 { "一" }
  case 2 { "二" }
  case 3 { "三" }
  case _ { "其他" }
};
debug_print(result);
```

`_` 是**弃元模式**，可以匹配任何值。当所有分支都不匹配时，`match` 表达式返回 `nil`。

## 字面量模式

匹配具体的值：

```mira
fn describe(value) {
  match value {
    case nil { "空值" }
    case true { "真" }
    case false { "假" }
    case 0 { "零" }
    case "" { "空字符串" }
    case _ { "其他: $value" }
  }
}

debug_print(describe(nil));
debug_print(describe(true));
debug_print(describe(0));
debug_print(describe(42));
```

## 变量模式

使用变量名捕获匹配到的值：

```mira
let data = 42;
match data {
  case x { debug_print("捕获到:", x) }
};
```

变量模式通常和其他模式组合使用，在后面的例子中会经常看到。

## 关系模式

使用比较运算符进行匹配：

```mira
fn classify_temp(temp) {
  match temp {
    case < 0 { "冰冻" }
    case < 15 { "寒冷" }
    case < 25 { "舒适" }
    case < 35 { "炎热" }
    case _ { "极热" }
  }
}

debug_print("10°C:", classify_temp(10));
debug_print("22°C:", classify_temp(22));
debug_print("38°C:", classify_temp(38));
```

支持的关系运算符：`>`、`>=`、`<`、`<=`、`==`、`!=`、`=~`、`!~`。

:::warning

与直接使用 [比较运算符](./05-booleans.md#比较运算符) 相比，在模式匹配中不会进行类型转换，类型不一致总是导致匹配失败。因此，不推荐在关系模式中使用 `!=` 和 `!~`，除非你明确知道左侧的值类型。

```mira
let v = '42';

debug_print(v != 42); // true，因为 '42' 和 42 类型不同
debug_print(v =~ 42); // true，隐式转换后 '42' 等于 42

debug_print(match v { case != 42 { true } case _ { false }}); // false，类型不一致导致匹配失败
debug_print(match v { case =~ 42 { true } case _ { false }}); // false，类型不一致导致匹配失败
```

:::

## 范围模式

使用 `..` 或 `..<` 匹配数值范围：

```mira
fn score_grade(score) {
  match score {
    case 90..100 { "优秀" }
    case 80..<90 { "良好" }
    case 60..<80 { "及格" }
    case 0..<60 { "不及格" }
    case _ { "无效分数" }
  }
}

debug_print("95 分:", score_grade(95));
debug_print("82 分:", score_grade(82));
debug_print("59 分:", score_grade(59));
```

## 数组模式

匹配数组结构并提取元素：

```mira
fn describe_array(arr) {
  match arr {
    case [] { "空数组" }
    case [only] { "单元素: $only" }
    case [a, b] { "两个元素: $a 和 $b" }
    case [first, ..rest] { "首元素 $first，还有 ${ len(rest) } 个" }
  }
}

debug_print(describe_array([]));
debug_print(describe_array([42]));
debug_print(describe_array([1, 2]));
debug_print(describe_array([1, 2, 3, 4, 5]));
```

`..rest` 会将剩余元素收集到一个数组中。它也可以出现在数组模式的中间或开头：

```mira
let arr = [1, 2, 3, 4, 5];
let [first, ..middle, last] = arr;
debug_print("首:", first);     // 1
debug_print("中间:", middle);  // [2, 3, 4]
debug_print("尾:", last);      // 5
```

## 记录模式

匹配记录结构并提取字段值：

```mira
fn greet_person(person) {
  match person {
    case (name: n, age: a) { "你好 $n，你 $a 岁了" }
    case (name: n) { "你好 $n" }
    case _ { "你好，陌生人" }
  }
}

debug_print(greet_person((name: "小明", age: 20)));
debug_print(greet_person((name: "小红")));
debug_print(greet_person(42));
```

### 简写形式

当变量名与键名相同时，可以用 `:name` 简写：

```mira
let person = (name: "小明", age: 20, city: "北京");
let (:name, :age) = person;
debug_print("$name，$age 岁");
```

### 嵌套解构

```mira
let data = (
  user: (name: "小明", age: 20),
  scores: [85, 92, 78],
);

let (user: (:name), scores: [first, ..]) = data;
debug_print("$name 的第一门成绩:", first);
```

## 逻辑组合模式

使用 `and`、`or`、`not` 组合多个模式：

```mira
fn classify(n) {
  match n {
    // 既大于 0 又小于 100
    case > 0 and < 100 { "0 到 100 之间" }
    // 等于 0 或等于 100
    case 0 or 100 { "边界值" }
    case _ { "范围外" }
  }
}

debug_print(classify(50));
debug_print(classify(0));
debug_print(classify(200));
```

```mira
// not 模式：匹配"不是某种模式"的值
fn not_nil(value) {
  match value {
    case not nil { "有值: $value" }
    case _ { "空" }
  }
}

debug_print(not_nil(42));
debug_print(not_nil(nil));
```

## Guard 条件

在模式后面添加 `if` 条件，进一步筛选：

```mira
fn fizzbuzz(n) {
  match n {
    case n if n % 15 == 0 { "FizzBuzz" }
    case n if n % 3 == 0 { "Fizz" }
    case n if n % 5 == 0 { "Buzz" }
    case n { "$n" }
  }
}

for i in 1..15 {
  debug_print("$(i):", fizzbuzz(i));
}
```

## `let` 中的解构

模式匹配不仅用于 `match`，也可以在 `let` 中使用，方便地提取数据：

```mira
// 数组解构
let [x, y, z] = [10, 20, 30];
debug_print("x=$x, y=$y, z=$z");

// 记录解构
let (name: name, age: age) = (name: "小明", age: 20);
debug_print("$name，$age 岁");

// 交换变量
let mut a = 1;
let mut b = 2;
debug_print("交换前: a=$a, b=$b");
(a, b) = (b, a);
debug_print("交换后: a=$a, b=$b");
```

## `is` 运算符

`is` 运算符用于在表达式中进行模式匹配，返回布尔值：

```mira
debug_print(42 is > 0);          // true
debug_print("hello" is "hello"); // true
debug_print(nil is nil);         // true
debug_print([1, 2] is [_, _]);   // true
debug_print(5 is 1..10);         // true
```

配合 `if` 使用：

```mira
let value = (name: "小明", age: 20);
if value is (name: _, age: > 18) {
  debug_print("成年人的记录");
}
```

## 实际应用

```mira
// 根据形状计算面积
fn area(shape) {
  match shape {
    case (type: "circle", r: r) {
      PI * r^2
    }
    case (type: "rectangle", w: w, h: h) {
      w * h
    }
    case (type: "triangle", base: b, height: h) {
      0.5 * b * h
    }
    case _ { nil }
  }
}

let shapes = [
  (type: "circle", r: 5),
  (type: "rectangle", w: 4, h: 6),
  (type: "triangle", base: 3, height: 8),
];

for shape in shapes {
  debug_print("$(shape.type) 面积:", area(shape));
}
```

## 小结

- `match` 表达式按顺序尝试各 `case` 分支
- 支持字面量、变量、关系、范围、数组、记录等多种模式
- `and`、`or`、`not` 组合复杂模式
- `if guard` 添加额外条件
- `let` 中可以使用模式解构
- `is` 运算符在表达式中进行模式匹配
