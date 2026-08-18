TODO:

1. 使用 `thiserror` 重构 `MiraError`，添加详细的错误类型，消除没必要的 `String`
2. 暂时移除 `MiraExtern`
3. `MiraAny` 类型重构：
   1. 改为 `MiraValue，``MiraAny = Option<MiraValue>`，使用 `None` 表示 `Uninitialized`
   2. 使用 nan-boxing，将尺寸控制为 8 byte
