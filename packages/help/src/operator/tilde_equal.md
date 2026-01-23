`=~` 是“近似/宽松相等”。常用于数值容差比较、字符串归一化比较等。

注意：对 `record` / `array` / `module` / `extern` 等类型可能不支持并抛出异常。

```mira
0.1 + 0.2 =~ 0.3; // true
"abc" =~ "ABC"; // true
1 =~ "1";        // true
```
