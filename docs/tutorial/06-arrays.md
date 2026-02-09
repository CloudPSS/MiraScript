# 数组

数组是一组有序的值的集合，是 MiraScript 中最常用的数据结构之一。

## 创建数组

使用方括号 `[]` 创建数组，元素之间用逗号分隔：

```mira
let numbers = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Charlie"];
let mixed = [1, "hello", true, nil]; // 可以混合不同类型

debug_print("数字:", numbers);
debug_print("名字:", names);
debug_print("混合:", mixed);
debug_print("空数组:", []);
```

### 范围填充

使用 `..` 或 `..<` 快速创建连续数字数组：

```mira
let a = [1..5];     // [1, 2, 3, 4, 5]，包含 5
let b = [1..<5];    // [1, 2, 3, 4]，不包含 5
let c = [0..4, 10..12]; // [0, 1, 2, 3, 4, 10, 11, 12]

debug_print("1..5:", a);
debug_print("1..<5:", b);
debug_print("组合范围:", c);
```

### 展开

使用 `..` 将一个数组展开到另一个数组中：

```mira
let first = [1, 2, 3];
let second = [4, 5, 6];
let combined = [..first, ..second];
let with_extra = [0, ..first, 99];

debug_print("合并:", combined);
debug_print("添加元素:", with_extra);
```

## 访问元素

### 索引访问

使用 `[索引]` 或 `.序数` 访问数组元素，索引从 `0` 开始：

```mira
let fruits = ["苹果", "香蕉", "橘子", "葡萄"];

debug_print("第一个:", fruits[0]);    // "苹果"
debug_print("第二个:", fruits.1);     // "香蕉"
debug_print("最后一个:", fruits[-1]); // "葡萄"，负索引从尾部开始
debug_print("倒数第二:", fruits[-2]); // "橘子"
```

访问不存在的索引会返回 `nil`，而不是报错：

```mira
let arr = [1, 2, 3];
debug_print("arr[10]:", arr[10]);  // nil
```

### 切片

使用范围获取数组的一部分（切片）：

```mira
let arr = [10, 20, 30, 40, 50];

debug_print("arr[1..3]:", arr[1..3]);    // [20, 30, 40]
debug_print("arr[1..<3]:", arr[1..<3]);  // [20, 30]
debug_print("arr[2..]:", arr[2..]);      // [30, 40, 50]，从索引 2 到末尾
debug_print("arr[..2]:", arr[..2]);      // [10, 20, 30]，从开头到索引 2
debug_print("arr[..]:", arr[..]);        // [10, 20, 30, 40, 50]，完整复制
debug_print("arr[1..-2]:", arr[1..-2]);  // [20, 30, 40]，支持负索引
```

## 不可变性

MiraScript 中数组是**不可变的**，创建后不能修改。要"修改"数组，需要创建一个新数组：

```mira
let original = [1, 2, 3];

// 不能直接修改：original[0] = 10; // 错误！

// 使用 with() 创建"修改"后的新数组
let modified = with(original, 0, 10);
debug_print("原数组:", original);   // [1, 2, 3] 不变
debug_print("新数组:", modified);   // [10, 2, 3]

// 也可以通过展开构造新数组
let appended = [..original, 4];
debug_print("追加元素:", appended); // [1, 2, 3, 4]
```

## 获取长度

```mira
let arr = [10, 20, 30];
debug_print("长度:", len(arr)); // 3
debug_print("空数组长度:", len([])); // 0
```

## 常用数组函数

### `map` —— 映射

对数组中每个元素应用一个函数，返回新数组：

```mira
let numbers = [1, 2, 3, 4, 5];
let doubled = map(numbers, fn { it * 2 });
debug_print("翻倍:", doubled); // [2, 4, 6, 8, 10]
```

### `filter` —— 过滤

保留满足条件的元素：

```mira
let numbers = [1, 2, 3, 4, 5, 6, 7, 8];
let evens = filter(numbers, fn { it % 2 == 0 });
debug_print("偶数:", evens); // [2, 4, 6, 8]
```

### `sort` —— 排序

```mira
let unsorted = [3, 1, 4, 1, 5, 9, 2, 6];
debug_print("升序:", sort(unsorted));

// 使用自定义比较函数降序排列
let desc = sort(unsorted, fn (a, b) { b - a });
debug_print("降序:", desc);
```

### `find` —— 查找

返回第一个满足条件的元素，找不到则返回 `nil`：

```mira
let numbers = [1, 3, 5, 8, 9, 12];
let first_even = find(numbers, fn { it % 2 == 0 });
debug_print("第一个偶数:", first_even); // 8
```

### `reverse` —— 反转

```mira
let arr = [1, 2, 3, 4, 5];
debug_print("反转:", reverse(arr)); // [5, 4, 3, 2, 1]
```

### `flatten` —— 扁平化

将嵌套数组展平：

```mira
let nested = [[1, 2], [3, 4], [5, 6]];
debug_print("展平:", flatten(nested)); // [1, 2, 3, 4, 5, 6]
```

### `all` 和 `any` —— 全部/任一满足

```mira
let numbers = [2, 4, 6, 8];
debug_print("全部是偶数:", all(numbers, fn { it % 2 == 0 })); // true
debug_print("存在大于 5:", any(numbers, fn { it > 5 }));       // true
```

## 检查元素是否存在

使用 `in` 运算符检查数组是否包含某个值：

```mira
let colors = ["red", "green", "blue"];
debug_print("包含 green:", "green" in colors);   // true
debug_print("包含 yellow:", "yellow" in colors); // false
```

## 实际应用

```mira
// 计算一组成绩的平均分
let scores = [85, 92, 78, 95, 88, 76, 90];
let avg = sum(..scores) / len(scores);
debug_print("平均分: $(avg:.1)");

// 找出及格的同学并排序
let students = [
  (name: "小明", score: 85),
  (name: "小红", score: 55),
  (name: "小华", score: 92),
  (name: "小李", score: 48),
  (name: "小张", score: 76),
];

let passed = filter(students, fn { it.score >= 60 });
let ranked = sort(passed, fn (a, b) { b.score - a.score });

debug_print("及格学生（按分数降序）:");
for s in ranked {
  debug_print("  $(s.name): $(s.score) 分");
}
```

## 小结

- 使用 `[]` 创建数组，`[1..5]` 快速生成连续数字
- 索引从 `0` 开始，负索引从尾部计算，越界返回 `nil`
- 数组**不可变**，通过 `with()` 或展开创建新数组
- `map`、`filter`、`sort`、`find` 等函数进行数据处理
- `in` 运算符检查元素是否存在
