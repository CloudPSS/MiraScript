---
title: 'in'
---

`in` 用于“包含/存在”判断，以及 `for` 循环。

---

对于 `array` 类型，`in` 用于判断某个元素是否在数组中存在：

```mira
fn is_spring(month) {
  month in [3, 4, 5]
}
```

---

对于 `record` / `module` / `extern` 类型，`in` 用于判断某个键是否在容器中存在：

```mira
let record = (a: 1, b: 2, c: 3);
let a_exists = 'a' in record; // true
```

---

也可与 `global` 关键字结合使用，判断全局变量是否存在：

```mira
if 'my_global' in global {
  my_global += 1;
}
```

---

同时，`for` 循环也使用 `in`：

```mira
for i in 1..3 {
  // i: 1, 2, 3
}
```
