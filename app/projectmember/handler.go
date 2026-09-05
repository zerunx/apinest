package projectmember

import (
	"nest-api/internal/utils"
	"nest-api/pkg/logger"
	"nest-api/pkg/response"
	"nest-api/pkg/validator"

	"github.com/gin-gonic/gin"
)

type Handler struct{}

func (*Handler) List(c *gin.Context) {
	var params ListRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	list, err := (Service{}).List(c.Request.Context(), utils.GetUserID(c), params)
	if err != nil {
		logger.Error("project member list failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, list)
}

func (*Handler) Add(c *gin.Context) {
	var params AddRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	if err := (Service{}).Add(c.Request.Context(), utils.GetUserID(c), params); err != nil {
		logger.Error("project member add failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, nil)
}

func (*Handler) Remove(c *gin.Context) {
	var params RemoveRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	if err := (Service{}).Remove(c.Request.Context(), utils.GetUserID(c), params); err != nil {
		logger.Error("project member remove failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, nil)
}
