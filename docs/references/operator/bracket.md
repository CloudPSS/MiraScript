---
title: '[ ]'
token: ['[', ']']
---

`[ ]` 用于数组字面量、索引访问与切片。

---

数组字面量：

```mira
let arr = [1, 2, 3];
```

---

索引访问：

```mira
let first = arr[0];
let last = arr[-1];
let value = record["key"];
```

---

数组切片：

```mira
let subarr1 = arr[0..1];   // 取前两个元素
let subarr2 = arr[..<-1];  // 取到倒数第一个元素（不含）
```
