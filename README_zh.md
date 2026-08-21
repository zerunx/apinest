# Apinest

> 开源 API 管理 / 接口文档协作平台

[English](README.md) | [中文](README_zh.md)

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

## 特性

- **Web 桌面双端同步**：Web 与桌面客户端共用同一套工作空间与接口数据
- **成员协作**：邀请加入工作空间，按角色控制编辑权限
- **桌面客户端**：本地同步与发请求，支持环境变量替换与请求历史
- **接口文档**：文件夹分层、接口 CRUD、请求/响应相关字段管理
- **环境与变量**：多环境配置，便于本地 / 测试 / 生产切换
- **项目分享**：生成分享记录，访客无需登录即可查看勾选接口
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
