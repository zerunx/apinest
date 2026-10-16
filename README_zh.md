# Apinest

> 开源 API 管理 / 接口文档协作平台

[English](README.md) | [中文](README_zh.md)

**Apinest**（ApiNest）是一套可自托管的 API 协作与调试平台，覆盖接口文档、环境配置与本地发请求。

## 主要特性

- **双端同步**：Web 与 Desktop 可同步，避免本地测试后需要再写一份文档
- **绝对安全**：请求在本机发出，密钥与接口明细不必依赖第三方云端
- **自主可控**：Docker 配置，账号、权限与数据都在自己服务器
- **开放互通**：支持 Postman、OpenApi、Eolink、YApi 等格式导入，以及 Apinest 自身导入导出
- **团队协作**：工作空间邀请与角色权限；分享链接可让访客免登录查看指定接口

## 展示

<table>
  <tr>
    <th align="center">Web</th>
    <th align="center">Desktop</th>
  </tr>
  <tr>
    <td width="50%" align="center">
      <img src="docs/images/web.png" alt="ApiNest Web" />
    </td>
    <td width="50%" align="center">
      <img src="docs/images/desktop.png" alt="ApiNest Desktop" />
    </td>
  </tr>
</table>

## 快速开始

镜像地址：[Docker Hub · zerunx/nest](https://hub.docker.com/r/zerunx/nest/tags)

### 1. 拉取镜像

```bash
docker pull zerunx/nest:latest
docker pull postgres:18.4-trixie
docker pull caddy:2
```

反向代理可用 Caddy 或 Nginx。下面以 Caddy 为例；若用 Nginx，不要再占用 80/443 端口。

### 2. 准备目录和 Caddyfile

路径需与下面 `docker-compose.yml` 中的挂载一致：

```bash
mkdir -p /data/docker/caddy /data/docker/postgres/data /data/docker/nest/runtime
```

创建 `/data/docker/caddy/Caddyfile`：

```caddyfile
:80 {
    reverse_proxy nest:3000
}
```

有域名时把 `:80` 换成 `your.domain.com`，Caddy 会自动申请 HTTPS。

若用 Nginx，将反代目标设为 `http://nest:3000`，并与 `nest` 容器加入同一 Docker 网络。

### 3. 创建网络

```bash
docker network create common-network
```

### 4. 编写 docker-compose.yml

```yaml
services:
  caddy:
    image: caddy:2
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /data/docker/caddy/Caddyfile:/etc/caddy/Caddyfile
      - /data/docker/caddy/data:/data
      - /data/docker/caddy/config:/config
    restart: unless-stopped

  postgres:
    image: postgres:18.4-trixie
    environment:
      TZ: Asia/Shanghai
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: 123123
    expose:
      - 5432
    volumes:
      - /data/docker/postgres/data:/var/lib/postgresql
    restart: unless-stopped

  nest:
    image: zerunx/nest:latest
    expose:
      - 3000
    volumes:
      - /data/docker/nest/runtime/:/app/runtime
    restart: unless-stopped
    depends_on:
      - postgres

networks:
  default:
    name: common-network
    external: true
```

### 5. 启动

```bash
docker compose up -d
```

浏览器访问服务器 IP（80 端口）或你的域名。`nest` 只在容器网络内监听 3000，请走 Caddy，不要访问 `IP:3000`。

### 完成安装

打开站点后按安装向导填写管理员账号和 PostgreSQL。若按上面的 Compose 启动，数据库可填：

- 主机：`postgres`
- 端口：`5432`
- 用户 / 密码：`postgres` / `123123`
- 数据库：`postgres`

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Go · Gin · Ent · JWT · Zap · WebSocket |
| 前端 | Vue 3 · TypeScript · Vite · Element Plus · Vue Router · Axios |
| 桌面端 | Tauri 2 · Rust · Vue 3 · SQLite |
| 基础设施 | Docker · PostgreSQL · Valkey · Caddy |

## 更多能力

- **接口文档**：文件夹分层、接口 CRUD、请求/响应字段与示例管理
- **桌面客户端**：环境变量替换、请求历史、文件夹级 Headers / Authorization 继承
- **环境与变量**：多套环境配置，便于本地 / 测试 / 生产切换
- **系统能力**：安装向导、用户鉴权、管理后台、中英双语

## 打包 Desktop

```bash
cd desktop
pnpm install
pnpm run tauri:build
```

产物在 `desktop/src-tauri/target/release/bundle/`：

- Windows：`nsis/*.exe`、`msi/*.msi`
- macOS：`macos/*.app`、`dmg/*.dmg`

## 路线图

- [x] desktop 端
- [x] 粘贴、复制 CURL
- [x] raw 采用 CodeMirror
- [x] ws grpc 协议
- [x] proto 文件导入参数，自动生成模拟参数
- [x] apinest 导入，导出
- [x] postman apifox eolink yapi 等接口格式导入
- [x] Web端接口查看的样式
- [ ] 内置 / 多数据库支持（当前仅外部 PostgreSQL）
- [x] 字典库 
- [ ] tcp udp 方式
- [ ] 请求接口的前置和后置脚本

## License

ApiNest is licensed under the [Apache License 2.0](LICENSE)
