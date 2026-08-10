# UnexpectedBreak

等级：**错误**

在循环之外出现意外的 `break`

## 如何修复

`break` 只能出现在 `loop`、`while` 或 `for` 循环中。

### 修改前

```mira
break;
```

### 修复后

```mira
loop {
  break;
}
```
