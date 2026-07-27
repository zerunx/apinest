# ApiNest Desktop

Tauri 2 + Vue 3 桌面客户端。服务端（ApiNest）为目录权威来源，客户端同步到本地 SQLite，并在本地发送 HTTP 请求。

## 功能

- 登录（JWT，对接 Nest `/api/v1/auth/login`）
- 工作空间 / 项目切换、左侧文件夹与接口树、搜索
- 右侧请求编辑与发送、响应查看、环境变量 `{{var}}` 替换
- 设置：中英文切换、测试连接、立即同步、清除本地缓存（不影响服务器）
- 请求历史写入本地库

## 开发

```bash
cd desktop
pnpm install
pnpm run tauri:dev
```

## 技术分工

| 层 | 技术 |
|----|------|
| UI | Vue 3 + TypeScript + Vue Router |
| 壳 / 本地库 / 发请求 | Rust（Tauri 2 + rusqlite + reqwest） |
| 业务真相 | 现有 ApiNest Go API |

本地数据库文件位于应用数据目录下的 `apinest-client.db`（硬盘持久化，非内存库）。

## License

ApiNest 采用 [Apache License 2.0](LICENSE) 开源。修改或再分发时须保留版权声明与许可证文本。