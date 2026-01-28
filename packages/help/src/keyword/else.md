---
token: 'else'
---

`else` 用于 `if` 表达式，也可用作 `while` 和 `for` 循环的可选分支。

---

作为 `if` 表达式的分支

```mira
let x = if false { 1 } else { 2 }; // 2
```

---

作为循环的可选分支

```mira
let mut i = 0;
let result = while i < 5 {
  i += 1;
  if i == 6 {
    // 使用 break 语句提前退出循环，并返回一个值
    break i * 2;
  }
} else {
  // 当循环正常结束时执行
  100
};
// result 的值是 100
```
