package bootstrap

import (
	"log"

	"nest-api/internal/redis"
	"nest-api/internal/runtime"
)

func InitRedis() {
	if !runtime.IsInstalled() {
		return
	}

	if err := redis.Init(); err != nil {
		log.Printf("redis unavailable, skip: %v", err)
	}
}
