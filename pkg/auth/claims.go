package auth

import jwtv5 "github.com/golang-jwt/jwt/v5"

type Claims struct {
	UserID int64 `json:"user_id"`
	Type   int8  `json:"type"` // 1: access, 2: refresh

	jwtv5.RegisteredClaims
}
