package user

import (
	"nest-api/pkg/logger"
	"nest-api/pkg/response"
	"nest-api/pkg/validator"

	"github.com/gin-gonic/gin"
)

type Handler struct {
}

func (*Handler) Create(c *gin.Context) {

	var params CreateRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	err := UserService{}.Create(c.Request.Context(), params)
	if err != nil {
		logger.Error("user create failed", err)
		response.Fail(c, err)
		return
	}

	response.Success(c, nil)
}

func (*Handler) List(c *gin.Context) {

	var params ListRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	list, meta, err := UserService{}.List(c.Request.Context(), params)
	if err != nil {
		logger.Error("user list failed", err)
		response.Fail(c, err)
		return
	}

	response.Paginate(c, list, meta)
}

func (*Handler) Items(c *gin.Context) {

	var params ListRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}
	list, err := UserService{}.Items(c.Request.Context(), params)
	if err != nil {
		logger.Error("user list failed", err)
		response.Fail(c, err)
		return
	}

	response.Success(c, list)
}

func (*Handler) Update(c *gin.Context) {

	var params UpdateRequest
	if err := validator.Bind(c, &params); err != nil {
		response.Fail(c, err)
		return
	}

	err := UserService{}.Update(c.Request.Context(), params)
	if err != nil {
		logger.Error("user update failed", err)
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

	err := UserService{}.Delete(c.Request.Context(), params)
	if err != nil {
		logger.Error("user delete failed", err)
		response.Fail(c, err)
		return
	}

	response.Success(c, nil)
}
