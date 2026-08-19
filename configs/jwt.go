package configs

import (
	"time"

	"nest-api/internal/utils"
	"nest-api/pkg/auth"
)

var JWT = auth.Config{
	Secret:        utils.String("JWT_SECRET", "your-jwt-secret"),
	AccessExpire:  24 * time.Hour,
	RefreshExpire: 7 * 24 * time.Hour,
}
