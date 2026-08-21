# ---- frontend ----
FROM node:22-bookworm AS web-builder

WORKDIR /web

COPY web/package.json web/package-lock.json ./
RUN npm ci

COPY web/ ./
RUN npm run build

# ---- backend ----
FROM golang:1.26.3-trixie AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o main ./cmd/api/main.go

# ---- runtime ----
FROM alpine:3.23

WORKDIR /app

RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /app/main .
COPY --from=web-builder /web/dist ./web/dist

ENV TZ=Asia/Shanghai \
    GIN_MODE=release \
    WEB_ROOT=/app/web/dist \
    LOG_TO_CONSOLE=true \
    LOG_TO_FILE=false

EXPOSE 3000

CMD ["./main"]
