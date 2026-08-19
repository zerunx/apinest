package ws

import (
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func Upgrade(c *gin.Context, sessionId string) error {

	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return err
	}

	client := &Client{
		SessionId: sessionId,
		Conn:      conn,
		ExpireAt:  time.Now().Add(5 * time.Minute),
	}

	Manager.Add(client)

	Manager.Success(client.SessionId, nil)

	go readLoop(client)
	go autoClose(client)

	return nil
}

func readLoop(client *Client) {

	defer func() {
		client.Conn.Close()
		Manager.Remove(client.SessionId)
	}()

	for {
		_, msg, err := client.Conn.ReadMessage()
		if err != nil {
			fmt.Println("disconnect:", err)
			break
		}
		fmt.Println("receive:", string(msg))
	}
}

func autoClose(client *Client) {
	time.Sleep(5 * time.Minute)
	fmt.Println("timeout close:", client.SessionId)
	client.Conn.Close()
	Manager.Remove(client.SessionId)
}
