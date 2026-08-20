# Apinest

> 开源 API 管理 / 接口文档协作平台 

[中文](README.md) | [English](README_en.md)

**Apinest**（ApiNest）是一套可自托管的 API 协作管理平台，用于集中管理接口文档，适合作为 Postman / Apifox 等等工具的轻量自托管替代方案。

## 它是做什么的

把团队放进同一个工作空间，按项目、文件夹组织接口，配置多套环境与变量，邀请成员共同编辑，再通过分享链接把指定接口开放给无需登录的访问者。

适合替代零散的文档表格或私有接口清单，让接口定义、环境切换与协作权限落在同一处。

## 在线演示

- 地址：[https://nest.lengyuye.top/login](https://nest.lengyuye.top/login)
- 账号：`admin`
- 密码：`123123`

## 快速开始

镜像地址：[Docker Hub · zerunx/nest](https://hub.docker.com/r/zerunx/nest/tags)

```bash
docker pull zerunx/nest:latest
```

> **部署前提**
>
> - 需自行准备可访问的 **PostgreSQL**（建议 14+）
> - 当前**不支持内置数据库**，连接信息在首次安装向导中填写，无需配置容器环境变量
> - 镜像已包含前端与后端，默认监听 **`3000`**
> - 请挂载 **`runtime`** 目录到宿主机，用于持久化安装配置

### Docker Compose

```yaml
services:
  nest:
    image: zerunx/nest:latest
    ports:
      - "3000:3000"
    volumes:
      - ./runtime:/app/runtime
    restart: unless-stopped
```

```bash
docker compose up -d
```

### Docker Run

```bash
docker run -d --name nest \
  -p 3000:3000 \
  -v ./runtime:/app/runtime \
  zerunx/nest:latest
```

### 反向代理

Nginx、Caddy 等只需把流量反代到 Apinest 服务（默认 `http://127.0.0.1:3000` 或容器网络中的 `nest:3000`），前后端同源，无需单独部署静态站点。

```caddyfile
your.domain.com {
    reverse_proxy nest:3000
}
```

### 完成安装

浏览器访问站点后按安装向导填写 PostgreSQL 连接信息与管理员账号，完成后即可登录使用。

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Go · Gin · Ent · JWT · Zap · WebSocket |
| 前端 | Vue 3 · TypeScript · Vite · Element Plus · Vue Router · Axios |
| 桌面端 | Tauri 2 · Rust · Vue 3 · SQLite |
| 基础设施 | Docker · PostgreSQL · Valkey · Caddy |

## 特性

- **工作空间与项目**：创建、重命名、删除；按项目归档接口资产
- **接口文档**：文件夹分层、接口 CRUD、请求/响应相关字段管理
- **环境与变量**：多环境配置，便于本地 / 测试 / 生产切换
- **成员协作**：邀请加入工作空间，按角色控制编辑权限
- **项目分享**：生成分享记录，访客无需登录即可查看勾选接口
- **桌面客户端**：本地同步与发请求，支持环境变量替换与请求历史
- **系统能力**：安装向导、用户鉴权、管理后台、中英双语

## 路线图

- [x] 桌面端对接
- [x] 粘贴、复制 CURL
- [x] mac 优化
- [x] raw 采用 CodeMirror
- [x] ws grpc 协议
- [x] proto 文件导入参数，自动生成模拟参数
- [x] apinest 导入，导出
- [ ] postman apifox eolink yapi 等接口格式导入
- [ ] Web端接口查看的样式
- [ ] 液态玻璃主题与边框细化
- [ ] 内置 / 多数据库支持（当前仅外部 PostgreSQL）

## License

ApiNest is licensed under the [Apache License 2.0](LICENSE)