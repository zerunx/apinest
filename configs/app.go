package configs

import (
	"nest-api/internal/utils"
)

type AppConfig struct {
	Server ServerConfig
}

type ServerConfig struct {
	Host string
	Port int
}

var App = AppConfig{
	Server: ServerConfig{
		Host: utils.String("APP_HOST", "0.0.0.0"),
		Port: utils.Int("APP_PORT", 3000),
	},
}
