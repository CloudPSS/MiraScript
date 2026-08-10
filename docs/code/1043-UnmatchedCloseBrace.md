# UnmatchedCloseBrace

等级：**错误**

发现未匹配的 `}`

## 如何修复

删除没有对应左花括号的 `}`，或补全前面的块。

### 修改前

```mira
let value = 1;
}
```

### 修复后

```mira
let value = 1;
```
