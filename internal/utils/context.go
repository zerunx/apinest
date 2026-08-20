package utils

import (
	"encoding/json"
	"strconv"

	"github.com/gin-gonic/gin"
)

const (
	ContextUserID = "user_id"
	ContextClaims = "claims"
)

func GetUserID(c *gin.Context) int64 {
	v, exists := c.Get(ContextUserID)
	if !exists {
		return 0
	}
	return toInt64(v)
}

func toInt64(v any) int64 {
	switch n := v.(type) {
	case int64:
		return n
	case int:
		return int64(n)
	case int32:
		return int64(n)
	case float64:
		return int64(n)
	case json.Number:
		i, _ := n.Int64()
		return i
	case string:
		i, _ := strconv.ParseInt(n, 10, 64)
		return i
	default:
		return 0
	}
}
