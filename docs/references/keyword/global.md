---
title: 'global'
---

`global` 关键字用于读取全局变量或判断某个全局变量是否存在。

---

当变量名与局部变量或关键字冲突时，可以使用 `global` 关键字来访问全局变量。

```mira
let some_name = 'shadow';
some_name; // 访问局部变量 'some_name'
global.some_name; // 访问全局变量 'some_name'
global.if; // 访问全局变量 'if'，而不是关键字 'if'
```

---

动态判断和访问全局变量：

```mira
fn get_global(name, fallback) {
  if name in global {
    global[name]
  } else {
    fallback
  }
}
```
