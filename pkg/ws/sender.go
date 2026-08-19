package ws

import (
	"errors"

	"github.com/bytedance/sonic"
	"github.com/gorilla/websocket"
)

// 对外方法
func (m *WsManager) Success(sessionId string, data interface{}) error {

	return m.sendMsg(sessionId, 0, "ok", data)

	// 发送完后删除ws链接
	// Manager.Remove(sessionId)
}

func (m *WsManager) Error(sessionId string, msg string) error {
	return m.sendMsg(sessionId, 500, msg, nil)
}

func (m *WsManager) sendMsg(sessionId string, code int, msg string, data interface{}) error {

	resp := Response{
		Code:    code,
		Message: msg,
		Data:    data,
	}

	bytes, err := sonic.Marshal(resp)
	if err != nil {
		return err
	}

	return m.send(sessionId, bytes)
}

func (m *WsManager) send(sessionId string, data []byte) error {

	client, ok := m.Get(sessionId)
	if !ok {
		return errors.New("connection not found")
	}

	err := client.Conn.WriteMessage(websocket.TextMessage, data)
	if err != nil {
		m.Remove(sessionId)
		return err
	}

	return nil
}
