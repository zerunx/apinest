package auth

import (
	"errors"
	"time"

	jwtv5 "github.com/golang-jwt/jwt/v5"
)

var (
	ErrInvalidSigningMethod = errors.New("invalid signing method")
	ErrInvalidToken         = errors.New("invalid token")
	ErrTokenExpired         = errors.New("token is expired")
	ErrWrongTokenType       = errors.New("wrong token type")
)

type JWT struct {
	secret        []byte
	accessExpire  time.Duration
	refreshExpire time.Duration
}

func New(cfg Config) *JWT {
	return &JWT{
		secret:        []byte(cfg.Secret),
		accessExpire:  cfg.AccessExpire,
		refreshExpire: cfg.RefreshExpire,
	}
}

func (j *JWT) GenerateToken(claims jwtv5.Claims) (string, error) {
	token := jwtv5.NewWithClaims(jwtv5.SigningMethodHS256, claims)
	return token.SignedString(j.secret)
}

func (j *JWT) ParseToken(tokenStr string, claims jwtv5.Claims) error {
	token, err := jwtv5.ParseWithClaims(tokenStr, claims,
		func(token *jwtv5.Token) (interface{}, error) {
			if _, ok := token.Method.(*jwtv5.SigningMethodHMAC); !ok {
				return nil, ErrInvalidSigningMethod
			}
			return j.secret, nil
		},
	)
	if err != nil {

		if errors.Is(err, jwtv5.ErrTokenExpired) {
			return ErrTokenExpired
		}
		return err
	}
	if !token.Valid {
		return ErrInvalidToken
	}
	return nil
}
