# 实战练习

恭喜你完成了 MiraScript 入门教程的学习！本章通过几个综合实例，帮助你巩固所学知识。

:::tip
每个示例都可以直接编辑和运行。建议先阅读代码，理解思路，然后尝试修改或扩展。
:::

## 示例一：统计分析

计算一组数据的均值、方差和标准差。

```mira
let data = [72, 85, 90, 68, 76, 93, 82, 88, 79, 95];

// 平均值
let n = len(data);
let avg = sum(..data) / n;

// 方差：每个值与均值之差的平方的平均
let mut variance = 0;
for x in data {
  variance += (x - avg)^2;
}
variance = variance / n;

// 标准差：方差的平方根
let std_dev = sqrt(variance);

debug_print("数据:", data);
debug_print("样本数:", n);
debug_print("均值: $(avg:.2)");
debug_print("方差: $(variance:.2)");
debug_print("标准差: $(std_dev:.2)");
```

## 示例二：斐波那契数列

用不同方式实现斐波那契数列，体会递归、循环和缓存的差异。

```mira
// 方式一：简单递归
fn fib_recursive(n) {
  if n <= 2 { n }
  else { fib_recursive(n - 1) + fib_recursive(n - 2) }
}

// 方式二：迭代
fn fib_iterative(n) {
  if n <= 2 { return n }
  let mut a = 1;
  let mut b = 2;
  for _ in 3..n {
    let temp = a + b;
    a = b;
    b = temp;
  }
  b
}

// 方式三：模式匹配 + 尾递归
fn fib_tail(n) {
  fn helper(n, a, b) {
    match n {
      case <= 2 { b }
      case _ { helper(n - 1, b, a + b) }
    }
  }
  if n <= 2 { n } else { helper(n, 1, 2) }
}

// 对比结果
for i in [1, 5, 10, 15] {
  debug_print("fib($i) = $(fib_recursive(i)) (递归) = $(fib_iterative(i)) (迭代) = $(fib_tail(i)) (尾递归)" );
}
```

## 示例三：数据处理管道

使用扩展调用 `::` 构建数据处理流水线。

```mira
// 学生成绩分析
let students = [
  (name: "张三", class: "A", math: 92, physics: 88),
  (name: "李四", class: "B", math: 78, physics: 85),
  (name: "王五", class: "A", math: 95, physics: 91),
  (name: "赵六", class: "B", math: 60, physics: 72),
  (name: "孙七", class: "A", math: 88, physics: 76),
  (name: "周八", class: "B", math: 73, physics: 68),
];

// 给每个学生计算总分
let with_total = students::map(fn {
  (..it, total: it.math + it.physics)
});

// 按总分排名
let ranked = with_total::sort(fn (a, b) { b.total - a.total });

debug_print("=== 成绩排名 ===");
let mut rank = 1;
for s in ranked {
  debug_print("第 $rank 名: $(s.name)（$(s.class)班）总分 $(s.total)");
  rank += 1;
}

// 按班级统计平均分
debug_print("\n=== 班级统计 ===");
for class in ["A", "B"] {
  let class_students = with_total::filter(fn { it.class == class });
  let class_avg = sum(..class_students::map(fn { it.total })) / len(class_students);
  debug_print("$class 班平均总分: $(class_avg:.1)");
}
```

## 示例四：矩阵运算

使用 `matrix` 模块实现线性代数运算。

```mira
// 定义两个矩阵
let a = [[1, 2, 3],
         [4, 5, 6]];

let b = [[7, 8],
         [9, 10],
         [11, 12]];

debug_print("A 的大小:", matrix.size(a));
debug_print("B 的大小:", matrix.size(b));

// 矩阵乘法 A(2×3) × B(3×2) = C(2×2)
let c = matrix.multiply(a, b);
debug_print("A × B =", c);

// 单位矩阵：任何矩阵乘以单位矩阵等于它自身
let eye = matrix.identity(2, 2);
let d = [[3, 1], [2, 4]];
debug_print("D × I =", matrix.multiply(d, eye));

// 转置
debug_print("A 的转置:", matrix.transpose(a));
```

## 示例五：回文检测

使用模式匹配实现字符串回文检测。

```mira
fn is_palindrome(text) {
  let ch = text::chars();

  fn check(arr) {
    match arr {
      // 空数组或单个字符：是回文
      case [] or [_] { true }
      // 首尾相同：去掉首尾继续检查
      case [first, ..middle, last] if first == last {
        check(middle)
      }
      // 首尾不同：不是回文
      case _ { false }
    }
  }

  check(ch)
}

let test_words = ["racecar", "hello", "madam", "level", "world", "noon"];
for word in test_words {
  let result = is_palindrome(word) ? "✓ 是" : "✗ 不是";
  debug_print("'$word' $result 回文");
}
```

## 示例六：简易排序算法

手动实现插入排序，加深对循环和数组操作的理解。

```mira
fn insertion_sort(arr) {
  let mut sorted = [];
  for item in arr {
    // 在 sorted 中找到合适的插入位置
    let pos = for i in 0..<len(sorted) {
      if item < sorted[i] {
        break i;
      }
    } else {
      len(sorted) // 没有找到更大的，插到末尾
    };

    // 构建插入后的新数组
    sorted = [..sorted[..<pos], item, ..sorted[pos..]];
  }
  sorted
}

let data = [38, 27, 43, 3, 9, 82, 10];
debug_print("排序前:", data);
debug_print("排序后:", insertion_sort(data));
```

## 下一步

你已经掌握了 MiraScript 的核心功能，可以继续探索：

- 查阅 [速查表](/cheatsheet/) 快速回顾语法
- 阅读 [语言规范](/spec/) 了解完整的语言细节
- 在实际项目中使用 MiraScript 编写脚本，实践出真知
