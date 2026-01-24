---
token: 'for'
---

## `for` 用于遍历可迭代对象。

遍历一个范围

```mira
let mut sum = 0;
for i in 1..3 {
  sum += i;
}
// sum == 6
```

---

遍历一个数组

```mira
let arr = [10, 20, 30];
let mut sum = 0;
for value in arr {
  sum += value;
}
// sum == 60
```

---

遍历一个记录

```mira
let record = (a: 1, b: 2, c: 3);
let mut sum = 0;
for key in record {
  sum += record[key];
}
// sum == 6
```
