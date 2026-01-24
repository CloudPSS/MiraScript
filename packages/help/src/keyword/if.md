---
token: 'if'
---

`if` 用于 `if` 表达式或 `match` 表达式的守卫条件。

---

`if` 表达式根据条件选择分支。条件必须是 `boolean`。

```mira
let x = if 1 > 0 { "pos" } else { "neg" };
```

---

使用 `else if` 添加多个条件分支：

```mira
let x = if 1 > 2 {
  "greater"
} else if 1 == 2 {
  "equal"
} else {
  "less"
};
```

---

在 `match` 表达式中使用 `if` 作为守卫条件：

```mira
let num = 10;
let result = match num {
  case n if n % 2 == 0 { "even" }
  case n { "odd" }
};
// result 的值是 "even"
```
