package router

import "github.com/gin-gonic/gin"

func Register(r *gin.Engine) {
	api := r.Group("/api")
	Install(api)
	Auth(api)
	Admin(api)
	User(api)
	Workspace(api)
	Member(api)
	Project(api)
	ProjectMember(api)
	Share(api)
	Folder(api)
	Interface(api)
	Environment(api)
	EnvVariable(api)
	ImportExport(api)
}
