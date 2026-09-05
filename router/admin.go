package router

import (
	"nest-api/app/admin"
	"nest-api/internal/middleware"

	"github.com/gin-gonic/gin"
)

func Admin(rg *gin.RouterGroup) {
	h := &admin.Handler{}
	r := rg.Group("/v1/admin")
	r.Use(middleware.JWTAuthMiddleware(), middleware.AdminRequired())
	{
		r.GET("/users", h.ListUsers)
		r.PUT("/users/:id", h.UpdateUser)
		r.DELETE("/users/:id", h.DeleteUser)
		r.GET("/workspaces", h.ListWorkspaces)
		r.PUT("/workspaces/:id/transfer", h.TransferWorkspace)
	}
}
