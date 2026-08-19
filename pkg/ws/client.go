package ws

import (
	"time"

	"github.com/gorilla/websocket"
)

type Client struct {
	SessionId string
	Conn      *websocket.Conn
	ExpireAt  time.Time
}
