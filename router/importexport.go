package router

import (
	"nest-api/app/importexport"
	"nest-api/internal/middleware"

	"github.com/gin-gonic/gin"
)

func ImportExport(rg *gin.RouterGroup) {
	h := &importexport.Handler{}
	r := rg.Group("/v1")
	r.Use(middleware.JWTAuthMiddleware())
	{
		r.GET("/import/export", h.Export)
		r.POST("/import/import", h.Import)
	}
}
