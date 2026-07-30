# ApiNest

> Open-source API collaboration platform (1.0.0)

[中文](README.md) | [English](README_en.md)

## What it does

ApiNest centralizes API documentation and team collaboration. Put your team in a shared workspace, organize APIs by project and folder, configure environments and variables, invite members to edit together, and share selected APIs via links that visitors can view without logging in.

It replaces scattered spreadsheets and private API lists, keeping API definitions, environment switching, and collaboration permissions in one place.

## Live demo

- URL: [https://nest.lengyuye.top/login](https://nest.lengyuye.top/login)
- Username: `admin`
- Password: `123123`

## Tech stack

| Layer | Technologies |
|------|------|
| Backend | Go · Gin · Ent · JWT · Zap · WebSocket |
| Frontend | Vue 3 · TypeScript · Vite · Element Plus · Vue Router · Axios |
| Infrastructure | Docker · PostgreSQL · Valkey · Caddy |

## Features

- **Workspaces & projects**: Create, rename, and delete; organize API assets by project
- **API docs**: Folder hierarchy, API CRUD, request/response field management
- **Environments & variables**: Multi-environment configs for local / staging / production
- **Team collaboration**: Invite members to a workspace with role-based edit permissions
- **Project sharing**: Share records so guests can view selected APIs without login
- **System**: Install wizard, auth, admin console (users & workspace transfer), zh/en UI

## Install

Image: [Docker Hub · zerunx/nest](https://hub.docker.com/r/zerunx/nest/tags)

```bash
docker pull zerunx/nest:latest

```

> **Before you deploy**
>
> - Prepare a reachable **PostgreSQL** instance (14+ recommended)
> - **No built-in database** — connection details are entered in the install wizard; no container env vars needed
> - The image includes frontend and backend; default port is **`3000`**
> - Mount a **`runtime`** volume to persist install configuration

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

### Reverse proxy

Point Nginx, Caddy, or similar at the ApiNest service only (default `http://127.0.0.1:3000`, or `nest:3000` on a Docker network). Frontend and API are same-origin; no separate static hosting is needed.

Caddy example:

```caddyfile
your.domain.com {
    reverse_proxy nest:3000
}
```

### Finish setup

Open the site in a browser and complete the install wizard (PostgreSQL connection + admin account). After that you can sign in and use the app.

## License

ApiNest is licensed under the [Apache License 2.0](LICENSE)
