package middleware

import (
	"strings"

	"nest-api/internal/auth"
	"nest-api/pkg/response"

	"github.com/gin-gonic/gin"
)

func JWTAuthMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {

		authHeader := c.GetHeader("Authorization")
		if authHeader == "" {
			response.Unauthorized(c)
			c.Abort()
			return
		}

		parts := strings.SplitN(authHeader, " ", 2)
		if len(parts) != 2 || parts[0] != "Bearer" {
			response.Unauthorized(c)
			c.Abort()
			return
		}

		tokenStr := parts[1]

		claims, err := auth.JWT.ParseAccessToken(tokenStr)
		if err != nil {
			response.Unauthorized(c)
			c.Abort()
			return
		}

		c.Set("user_id", claims.UserID)
		// c.Set("claims", claims)

		c.Next()
	}
}
