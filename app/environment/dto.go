package environment

type Item struct {
	ID        int64  `json:"id"`
	ProjectID int64  `json:"project_id"`
	Name      string `json:"name"`
	Remark    string `json:"remark"`
	IsDefault bool   `json:"is_default"`
	CreatedAt string `json:"created_at"`
}

type ListRequest struct {
	WorkspaceID int64 `form:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" binding:"required,min=1"`
}

type CreateRequest struct {
	WorkspaceID int64  `json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64  `json:"project_id" binding:"required,min=1"`
	Name        string `json:"name" binding:"required,min=1,max=100"`
	Remark      string `json:"remark" binding:"max=1024"`
	IsDefault   bool   `json:"is_default"`
}

type UpdateRequest struct {
	WorkspaceID   int64  `json:"workspace_id" binding:"required,min=1"`
	ProjectID     int64  `json:"project_id" binding:"required,min=1"`
	EnvironmentID int64  `json:"environment_id" binding:"required,min=1"`
	Name          string `json:"name" binding:"required,min=1,max=100"`
	Remark        string `json:"remark" binding:"max=1024"`
	IsDefault     bool   `json:"is_default"`
}

type DeleteRequest struct {
	WorkspaceID   int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID     int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	EnvironmentID int64 `form:"environment_id" json:"environment_id" binding:"required,min=1"`
}
