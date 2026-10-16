package workspace

import (
	"nest-api/internal/utils"
	"nest-api/pkg/logger"
	"nest-api/pkg/response"
	"nest-api/pkg/validator"

	"github.com/gin-gonic/gin"
)

type Handler struct{}

func (*Handler) Create(c *gin.Context) {
	var params CreateRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	id, err := (Service{}).Create(c.Request.Context(), utils.GetUserID(c), params)
	if err != nil {
		logger.Error("workspace create failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, gin.H{"id": id})
}

func (*Handler) List(c *gin.Context) {
	list, err := Service{}.List(c.Request.Context(), utils.GetUserID(c))
	if err != nil {
		logger.Error("workspace list failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, list)
}

func (*Handler) Get(c *gin.Context) {
	var params GetRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	data, err := Service{}.Get(c.Request.Context(), utils.GetUserID(c), params)
	if err != nil {
		logger.Error("workspace get failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, data)
}

func (*Handler) Update(c *gin.Context) {
	var params UpdateRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	if err := (Service{}).Update(c.Request.Context(), utils.GetUserID(c), params); err != nil {
		logger.Error("workspace update failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, nil)
}

func (*Handler) Delete(c *gin.Context) {
	var params DeleteRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	if err := (Service{}).Delete(c.Request.Context(), utils.GetUserID(c), params); err != nil {
		logger.Error("workspace delete failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, nil)
}

func (*Handler) TransferOwner(c *gin.Context) {
	var params TransferOwnerRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	if err := (Service{}).TransferOwner(c.Request.Context(), utils.GetUserID(c), params); err != nil {
		logger.Error("workspace transfer owner failed", err)
		response.Fail(c, err)
		return
	}
	response.Success(c, nil)
}
