# 函数

函数是 MiraScript 中组织和复用代码的基本方式。

## 声明函数

使用 `fn` 关键字声明函数。函数体是一个块表达式，最后一个表达式的值就是函数的返回值：

```mira
fn greet(name) {
  "你好，$name！"
}

debug_print(greet("小明"));
debug_print(greet("小红"));
```

### 多参数函数

```mira
fn add(x, y) {
  x + y
}

fn distance(x1, y1, x2, y2) {
  sqrt((x2 - x1)^2 + (y2 - y1)^2)
}

debug_print("3 + 4 =", add(3, 4));
debug_print("距离:", distance(0, 0, 3, 4));
```

### 单参数简写

当函数只有一个参数时，可以省略参数列表，该参数自动命名为 `it`：

```mira
fn double { it * 2 }
fn square { it^2 }
fn is_even { it % 2 == 0 }

debug_print("double(5) =", double(5));
debug_print("square(4) =", square(4));
debug_print("is_even(6) =", is_even(6));
```

:::tip
`it` 是 MiraScript 中非常实用的特性，在与 `map`、`filter` 等函数配合时尤其简洁。
:::

## 函数表达式

函数也可以作为值赋给变量。这种写法称为**函数表达式**或**匿名函数**：

```mira
let multiply = fn (x, y) { x * y };
let cube = fn { it^3 };

debug_print("3 × 4 =", multiply(3, 4));
debug_print("3³ =", cube(3));
```

:::info 函数声明 vs 函数表达式
函数声明 `fn name(...)` 会被**提升**，可以在声明之前调用。函数表达式没有这个特性：

```mira
// 函数声明可以在定义之前调用
debug_print(say_hello());

fn say_hello() { "Hello!" }
```

:::

## `return` 语句

函数体中最后一个表达式的值会自动作为返回值。也可以使用 `return` 提前返回：

```mira
fn classify(n) {
  if n > 0 {
    return "正数";
  }
  if n < 0 {
    return "负数";
  }
  "零"  // 最后一个表达式，自动返回
}

debug_print(classify(5));
debug_print(classify(-3));
debug_print(classify(0));
```

## 高阶函数

函数可以作为参数传递给其他函数，也可以作为返回值。这种使用方式称为**高阶函数**：

### 函数作为参数

```mira
fn apply_twice(func, value) {
  func(func(value))
}

fn add_one { it + 1 }
fn double { it * 2 }

debug_print("add_one 两次:", apply_twice(add_one, 5));   // 7
debug_print("double 两次:", apply_twice(double, 3));     // 12
```

### 与 `map`、`filter` 配合

匿名函数 + `it` 让数据处理代码非常简洁：

```mira
let numbers = [1, 2, 3, 4, 5, 6, 7, 8];

// 过滤偶数，然后翻倍
let result = filter(numbers, fn { it % 2 == 0 });
let doubled = map(result, fn { it * 2 });
debug_print("结果:", doubled);
```

## 展开参数

### 收集参数

使用 `..` 将多余的参数收集为数组：

```mira
fn print_all(first, ..rest) {
  debug_print("第一个:", first);
  debug_print("其余:", rest);
}

print_all("a", "b", "c", "d");
```

### 展开数组为参数

调用函数时，用 `..` 将数组展开为多个参数：

```mira
fn add(x, y) { x + y }

let pair = [3, 4];
debug_print("展开调用:", add(..pair)); // 等价于 add(3, 4)
```

## 默认参数值

MiraScript 没有专门的默认参数语法，但可以用 `??` 空合并运算符实现：

```mira
fn greet(name, greeting) {
  let g = greeting ?? "你好";
  "$g，$name！"
}

debug_print(greet("小明", "早上好"));
debug_print(greet("小红"));            // greeting 为 nil，使用默认值
```

## 闭包

函数可以捕获外层作用域中的变量，这称为**闭包**：

```mira
fn create_counter(start) {
  let mut count = start;
  fn {
    count += 1;
    count
  }
}

let counter = create_counter(0);
debug_print(counter()); // 1
debug_print(counter()); // 2
debug_print(counter()); // 3

// 每个计数器是独立的
let another = create_counter(100);
debug_print(another()); // 101
debug_print(counter()); // 4，不受影响
```

### 函数工厂

利用闭包可以创建"定制"的函数：

```mira
fn create_multiplier(factor) {
  fn { it * factor }
}

let times3 = create_multiplier(3);
let times5 = create_multiplier(5);

debug_print("times3(4) =", times3(4));   // 12
debug_print("times5(4) =", times5(4));   // 20
```

## 递归

函数可以调用自身，称为**递归**：

```mira
fn factorial(n) {
  if n <= 1 { 1 }
  else { n * factorial(n - 1) }
}

debug_print("5! =", factorial(5));   // 120
debug_print("10! =", factorial(10)); // 3628800
```

### 经典示例：斐波那契数列

```mira
fn fib(n) {
  if n <= 2 { n }
  else { fib(n - 1) + fib(n - 2) }
}

for i in 1..10 {
  debug_print("fib($i) =", fib(i));
}
```

## 实际应用

```mira
// 使用高阶函数进行数据处理
let students = [
  (name: "小明", score: 85),
  (name: "小红", score: 92),
  (name: "小华", score: 78),
  (name: "小李", score: 95),
  (name: "小张", score: 60),
];

// 找出优秀学生（90分以上）的名字
let excellent = filter(students, fn { it.score >= 90 });
let names = map(excellent, fn { it.name });
debug_print("优秀学生:", names);

// 计算平均分
let total = map(students, fn { it.score });
let avg = sum(..total) / len(total);
debug_print("平均分: $(avg:.1)");
```

## 小结

- `fn name(params) { body }` 声明函数，最后一个表达式是返回值
- 单参数函数可省略参数列表，参数名为 `it`
- 函数可以作为值传递（高阶函数）
- `..` 用于收集和展开参数
- 闭包可以捕获外层变量
- 函数可以递归调用自身
