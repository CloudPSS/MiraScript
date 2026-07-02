# 扩展调用与数据管道

当多个函数嵌套调用时，代码会变得难以阅读。MiraScript 提供了扩展调用运算符 `::` 来解决这个问题。

## 扩展调用运算符 `::`

`::` 运算符将左侧的值作为右侧函数的**第一个参数**传入：

```mira
fn double { it * 2 }
fn add(x, y) { x + y }

// 这两种写法等价
debug_print(double(5));
debug_print(5::double());

// 带参数时
debug_print(add(3, 4));
debug_print(3::add(4));
```

## 链式调用

`::` 最大的价值在于可以将多个操作**链式**组合，代码像流水线一样从左到右阅读：

### 嵌套调用 vs 链式调用

假设我们要对一组数据依次进行过滤、映射和排序：

```mira
let data = [5, 2, 8, 1, 9, 3, 7, 4, 6];

// 嵌套调用：需要从内向外阅读，不够直观
let result1 = sort(map(filter(data, fn { it > 3 }), fn { it * 10 }));

// 链式调用：从左到右，一目了然
let result2 = data
  ::filter(fn { it > 3 })
  ::map(fn { it * 10 })
  ::sort();

debug_print("嵌套调用:", result1);
debug_print("链式调用:", result2);
```

链式调用形式清晰地展示了数据处理的步骤：  
`data` → 过滤出大于 3 的 → 乘以 10 → 排序

### 字符串处理管道

```mira
let raw = "  Hello, World!  ";

let result = raw
  ::trim()
  ::replace("World", "MiraScript")
  ::split(", ")
  ::join(" & ");

debug_print(result);
```

## 与匿名函数配合

`::` 也可以与匿名函数配合。当右侧是一个函数表达式时，将其作为函数调用：

```mira
let result = 42
  ::(fn { it * 2 })()
  ::(fn { "结果是 $it" })();

debug_print(result);
```

## 实际应用

### 统计分析

```mira
let scores = [85, 92, 78, 95, 88, 67, 73, 91, 82, 76];

// 统计 80 分以上的人数和平均分
let high_scores = scores::filter(fn { it >= 80 });
let count = len(high_scores);
let avg = sum(..high_scores) / count;

debug_print("80分以上: $count 人");
debug_print("平均分: $(avg:.1)");
```

### 数据转换管道

```mira
let students = [
  (name: "小明", score: 85),
  (name: "小红", score: 92),
  (name: "小华", score: 78),
  (name: "小李", score: 95),
  (name: "小张", score: 60),
];

// 找出高分学生并格式化
let report = students
  ::filter(fn { it.score >= 80 })
  ::sort(fn (a, b) { b.score - a.score })
  ::map(fn { "$(it.name): $(it.score)分" })
  ::join(", ");

debug_print("优秀学生: $report");
```

### 数学运算管道

```mira
fn square { it^2 }
fn add_one { it + 1 }
fn halve { it / 2 }

// 4 → 平方(16) → 加1(17) → 减半(8.5)
let result = 4::square()::add_one()::halve();
debug_print("计算结果:", result);
```

### 文本词频统计

```mira
let text = "the quick brown fox jumps over the lazy dog the fox";

let words = text::split(" ");
let mut freq = ();

for word in words {
  let count = freq[word] ?? 0;
  freq = (..freq, `$word`: count + 1);
}

debug_print("词频统计:");
for key in freq {
  debug_print("  $key: $(freq[key])");
}
```

## 小结

- `value::func(args)` 等价于 `func(value, args)`
- 用 `::` 链式组合多个操作，代码从左到右阅读
- 特别适合与 `map`、`filter`、`sort` 等函数配合使用
- 将嵌套调用转为管道式写法，大幅提升可读性
