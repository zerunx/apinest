package ws

import "sync"

type WsManager struct {
	clients map[string]*Client
	lock    sync.RWMutex
}

var Manager = &WsManager{
	clients: make(map[string]*Client),
}

func (m *WsManager) Add(client *Client) {
	m.lock.Lock()
	defer m.lock.Unlock()
	m.clients[client.SessionId] = client
}

func (m *WsManager) Remove(sessionId string) {
	m.lock.Lock()
	defer m.lock.Unlock()
	delete(m.clients, sessionId)
}

func (m *WsManager) Get(sessionId string) (*Client, bool) {
	m.lock.RLock()
	defer m.lock.RUnlock()
	client, ok := m.clients[sessionId]
	return client, ok
}
