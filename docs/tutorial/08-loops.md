# 循环

循环用于重复执行一段代码。MiraScript 提供了三种循环结构，且它们都是**表达式**，有返回值。

## `for` 循环

`for` 循环用于遍历范围、数组或记录中的每个元素：

### 遍历范围

使用 [`..`](../references/operator/spread_range.md) 或 [`..<`](../references/operator/half_open_range.md) 指定数字范围：

```mira
// 1 到 5（包含 5）
for i in 1..5 {
  debug_print("i =", i);
}
```

```mira
// 0 到 4（不包含 5）
for i in 0..<5 {
  debug_print("i =", i);
}
```

### 遍历数组

```mira
let fruits = ["苹果", "香蕉", "橘子"];
for fruit in fruits {
  debug_print("水果:", fruit);
}
```

### 遍历记录

遍历记录时，循环变量是每个键名：

```mira
let person = (name: "小明", age: 20, city: "北京");
for key in person {
  debug_print("$key:", person[key]);
}
```

## `while` 循环

`while` 循环在条件为 `true` 时重复执行。和 `if` 一样，条件必须是布尔值：

```mira
let mut count = 1;
while count <= 5 {
  debug_print("第 $count 次");
  count += 1;
}
```

### 应用示例：累加求和

```mira
let mut sum = 0;
let mut i = 1;
while i <= 100 {
  sum += i;
  i += 1;
}
debug_print("1 + 2 + ... + 100 =", sum);
```

## `loop` 循环

`loop` 是无限循环，必须通过 `break` 退出：

```mira
let mut n = 1;
loop {
  if n > 5 {
    break;
  }
  debug_print("n =", n);
  n += 1;
}
```

## `break` 和 `continue`

- `break` —— 退出循环
- `continue` —— 跳过本次循环，进入下一次

```mira
// 使用 continue 跳过偶数
for i in 1..10 {
  if i % 2 == 0 {
    continue; // 跳过偶数
  }
  debug_print("奇数:", i);
}
```

```mira
// 使用 break 提前退出
for i in 1..100 {
  if i > 5 {
    break; // 只处理前 5 个
  }
  debug_print("处理:", i);
}
```

## 循环作为表达式

在 MiraScript 中，循环是表达式，可以通过 `break` 返回一个值。

### `break` 带返回值

```mira
let numbers = [3, 7, 2, 8, 1, 9, 4];

// 查找第一个大于 5 的数字
let found = for n in numbers {
  if n > 5 {
    break n;
  }
};

debug_print("找到:", found); // 7
```

### `else` 分支

当循环**正常结束**（没有被 `break` 退出）时，执行 `else` 分支：

```mira
// 查找偶数，如果没找到返回默认值
let result = for n in [1, 3, 5, 7] {
  if n % 2 == 0 {
    break n;
  }
} else {
  "没有找到偶数"
};

debug_print(result); // "没有找到偶数"
```

```mira
// while 也支持 else
let mut count = 0;
let msg = while count < 3 {
  count += 1;
} else {
  "循环正常结束"
};
debug_print(msg); // "循环正常结束"
```

## 实际应用

### 找出数组中的最大值

```mira
let data = [34, 12, 67, 23, 89, 45, 56];
let mut max_val = data[0];
for val in data {
  if val > max_val {
    max_val = val;
  }
}
debug_print("最大值:", max_val);
```

### 构建新数组

由于数组不可变，构建新数组需要逐步追加元素：

```mira
// 筛选出正数并求平方
let numbers = [-3, 1, -2, 4, 0, 5, -1];
let mut squares = [];
for n in numbers {
  if n > 0 {
    squares = [..squares, n^2];
  }
}
debug_print("正数的平方:", squares);
```

### 在矩阵中查找元素

```mira
fn find_in_matrix(matrix, target) {
  for row in 0..<len(matrix) {
    for col in 0..<len(matrix[row]) {
      if matrix[row][col] == target {
        return (row: row, col: col);
      }
    }
  }
  nil
}

let matrix = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9],
];

let pos = find_in_matrix(matrix, 5);
debug_print("5 的位置:", pos);
```

## 小结

- `for item in collection { }` 遍历数组、记录或范围
- `while condition { }` 条件循环
- `loop { }` 无限循环，用 `break` 退出
- `break value` 使循环返回一个值
- `else` 分支在循环正常结束时执行
- `continue` 跳过本次循环
