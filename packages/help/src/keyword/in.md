---
token: 'in'
order: 8
---

`in` 用于“包含/存在”判断：`key in container`。

- 常见容器：`record` / `module` / `extern` / `global`
- 其他类型通常返回 `false`

同时，`for` 循环也使用 `in`：`for x in iterable { ... }`。

```mira
t_true('x' in (x: 1));

for i in 1..3 {
  // i: 1, 2, 3
}
```
