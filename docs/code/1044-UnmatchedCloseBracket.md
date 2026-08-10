# UnmatchedCloseBracket

等级：**错误**

发现未匹配的 `]`

## 如何修复

删除没有对应左中括号的 `]`，或补全数组/索引表达式。

### 修改前

```mira
let value = 1;
]
```

### 修复后

```mira
let values = [1];
```
