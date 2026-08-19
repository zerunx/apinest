package auth

import (
	"time"

	jwtv5 "github.com/golang-jwt/jwt/v5"
)

const (
	TokenTypeAccess  = 1
	TokenTypeRefresh = 2
)

func (j *JWT) GenerateAccessToken(userID int64) (string, error) {
	claims := &Claims{
		UserID: userID,
		Type:   TokenTypeAccess,
		RegisteredClaims: jwtv5.RegisteredClaims{
			ExpiresAt: jwtv5.NewNumericDate(time.Now().Add(j.accessExpire)),
		},
	}
	return j.GenerateToken(claims)
}

func (j *JWT) GenerateRefreshToken(userID int64) (string, error) {
	claims := &Claims{
		UserID: userID,
		Type:   TokenTypeRefresh,
		RegisteredClaims: jwtv5.RegisteredClaims{
			ExpiresAt: jwtv5.NewNumericDate(time.Now().Add(j.refreshExpire)),
		},
	}
	return j.GenerateToken(claims)
}

func (j *JWT) GenerateTokenPair(userID int64) (access, refresh string, err error) {

	access, err = j.GenerateAccessToken(userID)
	if err != nil {
		return "", "", err
	}
	refresh, err = j.GenerateRefreshToken(userID)
	if err != nil {
		return "", "", err
	}
	return
}

func (j *JWT) ParseAccessToken(tokenStr string) (*Claims, error) {
	claims := &Claims{}
	if err := j.ParseToken(tokenStr, claims); err != nil {
		return nil, err
	}
	if claims.Type != TokenTypeAccess {
		return nil, ErrWrongTokenType
	}
	return claims, nil
}

func (j *JWT) ParseRefreshToken(tokenStr string) (*Claims, error) {
	claims := &Claims{}
	if err := j.ParseToken(tokenStr, claims); err != nil {
		return nil, err
	}
	if claims.Type != TokenTypeRefresh {
		return nil, ErrWrongTokenType
	}
	return claims, nil
}
