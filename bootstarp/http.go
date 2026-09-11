package bootstrap

import (
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"nest-api/configs"
	"nest-api/internal/middleware"
	"nest-api/internal/runtime"
	"nest-api/internal/utils"
	"nest-api/pkg/logger"
	"nest-api/pkg/response"
	"nest-api/router"

	"github.com/gin-gonic/gin"
)

func InitHTTP() {
	gin.SetMode(utils.String("GIN_MODE", gin.ReleaseMode))

	r := gin.New()

	_ = r.SetTrustedProxies(nil)

	r.Use(
		gin.Logger(),
		middleware.CORS(),
		middleware.Recovery(),
	)

	r.HandleMethodNotAllowed = true

	r.NoMethod(func(c *gin.Context) {
		response.MethodNotAllowed(c)
	})

	router.Register(r)

	webRoot := utils.String("WEB_ROOT", "/app/web/dist")
	if info, err := os.Stat(webRoot); err == nil && info.IsDir() {
		r.NoRoute(spaFallback(webRoot))
	} else {
		r.NoRoute(func(c *gin.Context) {
			response.NotFound(c)
		})
	}

	addr := fmt.Sprintf(
		"%s:%d",
		configs.App.Server.Host,
		configs.App.Server.Port,
	)

	logger.Info("HTTP Server Starting", "addr", addr, "installed", runtime.IsInstalled(), "web_root", webRoot)

	if err := r.Run(addr); err != nil {
		logger.Error("HTTP Server Stopped", err)
	}
}

func spaFallback(webRoot string) gin.HandlerFunc {
	fileServer := http.FileServer(http.Dir(webRoot))

	return func(c *gin.Context) {
		if strings.HasPrefix(c.Request.URL.Path, "/api") {
			response.NotFound(c)
			return
		}

		rel := strings.TrimPrefix(filepath.Clean(c.Request.URL.Path), string(filepath.Separator))
		full := filepath.Join(webRoot, rel)
		if info, err := os.Stat(full); err == nil && !info.IsDir() {
			fileServer.ServeHTTP(c.Writer, c.Request)
			return
		}

		c.File(filepath.Join(webRoot, "index.html"))
	}
}
