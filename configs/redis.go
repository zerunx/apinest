package configs

import "nest-api/internal/utils"

type RedisConfig struct {
	Host     string
	Port     int
	Password string
	DB       int
}

var Redis = RedisConfig{
	Host:     utils.String("REDIS_HOST", "redis"),
	Port:     utils.Int("REDIS_PORT", 6379),
	Password: utils.String("REDIS_PASSWORD", ""),
	DB:       utils.Int("REDIS_DB", 0),
}
