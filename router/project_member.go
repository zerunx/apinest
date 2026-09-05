package router

import (
	"nest-api/app/projectmember"
	"nest-api/internal/middleware"

	"github.com/gin-gonic/gin"
)

func ProjectMember(rg *gin.RouterGroup) {
	h := &projectmember.Handler{}
	r := rg.Group("/v1")
	r.Use(middleware.JWTAuthMiddleware())
	{
		r.GET("/project-members", h.List)
		r.POST("/project-members", h.Add)
		r.DELETE("/project-members", h.Remove)
	}
}
