# UnexpectedContinue

等级：**错误**

在循环之外出现意外的 `continue`

## 如何修复

`continue` 只能出现在循环中，用于开始下一次迭代。

### 修改前

```mira
continue;
```

### 修复后

```mira
for value in [1, 2] {
  if value == 1 { continue; }
}
```
