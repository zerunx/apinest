package middleware

import (
	"nest-api/pkg/logger"
	"nest-api/pkg/response"

	"github.com/gin-gonic/gin"
)

func Recovery() gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if err := recover(); err != nil {
				// stack := debug.Stack() // 堆栈错误信息，语法等错误
				logger.Error("error in interception system",
					"error", err,
					// "stack", string(stack),
				)
				response.Fail(c, nil)
				c.Abort()
			}
		}()
		c.Next()
	}
}
