package router

import (
	"nest-api/app/user"

	"github.com/gin-gonic/gin"
)

func User(rg *gin.RouterGroup) {

	user := &user.Handler{}
	r := rg.Group("/v1")
	{
		r.POST("/users", user.Create)
		r.GET("/users/items", user.Items)
		r.GET("/users", user.List)
		r.PUT("/users", user.Update)
		r.DELETE("/users", user.Delete)
	}
}
