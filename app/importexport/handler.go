package importexport

import (
	"nest-api/internal/utils"
	"nest-api/pkg/logger"
	"nest-api/pkg/response"
	"nest-api/pkg/validator"

	"github.com/gin-gonic/gin"
)

type Handler struct{}

func (*Handler) Export(c *gin.Context) {
	var params ExportRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	data, err := (Service{}).Export(c.Request.Context(), utils.GetUserID(c), params)
	if err != nil {
		logger.Error("importexport export failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, data)
}

func (*Handler) Import(c *gin.Context) {
	var params ImportRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	data, err := (Service{}).Import(c.Request.Context(), utils.GetUserID(c), params)
	if err != nil {
		logger.Error("importexport import failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, data)
}
