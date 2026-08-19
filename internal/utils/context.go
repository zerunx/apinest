package utils

import "github.com/gin-gonic/gin"

const (
	ContextUserID = "user_id"
	ContextClaims = "claims"
)

func GetUserID(c *gin.Context) int64 {

	v, exists := c.Get(ContextUserID)
	if !exists {
		return 0
	}

	id, ok := v.(int64)
	if !ok {
		return 0
	}

	return id
}
