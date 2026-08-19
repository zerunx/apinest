package auth

import "time"

type Config struct {
	Secret        string
	AccessExpire  time.Duration
	RefreshExpire time.Duration
}
