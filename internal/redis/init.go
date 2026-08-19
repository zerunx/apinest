package redis

import (
	"context"
	"fmt"

	"nest-api/configs"

	goredis "github.com/redis/go-redis/v9"
)

var Redis *goredis.Client

func Init() error {

	Redis = goredis.NewClient(&goredis.Options{
		Addr: fmt.Sprintf(
			"%s:%d",
			configs.Redis.Host,
			configs.Redis.Port,
		),
		Password: configs.Redis.Password,
		DB:       configs.Redis.DB,
	})

	return Redis.Ping(context.Background()).Err()
}
