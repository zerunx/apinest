package envvariable

type Item struct {
	ID            int64  `json:"id"`
	EnvironmentID int64  `json:"environment_id"`
	Key           string `json:"key"`
	Value         string `json:"value"`
	Description   string `json:"description"`
	CreatedAt     string `json:"created_at"`
	UpdatedAt     string `json:"updated_at"`
}

type ListRequest struct {
	WorkspaceID   int64 `form:"workspace_id" binding:"required,min=1"`
	ProjectID     int64 `form:"project_id" binding:"required,min=1"`
	EnvironmentID int64 `form:"environment_id" binding:"required,min=1"`
}

type CreateRequest struct {
	WorkspaceID   int64  `json:"workspace_id" binding:"required,min=1"`
	ProjectID     int64  `json:"project_id" binding:"required,min=1"`
	EnvironmentID int64  `json:"environment_id" binding:"required,min=1"`
	Key           string `json:"key" binding:"required,min=1,max=200"`
	Value         string `json:"value" binding:"max=2000"`
	Description   string `json:"description" binding:"max=1024"`
}

type UpdateRequest struct {
	WorkspaceID   int64  `json:"workspace_id" binding:"required,min=1"`
	ProjectID     int64  `json:"project_id" binding:"required,min=1"`
	EnvironmentID int64  `json:"environment_id" binding:"required,min=1"`
	VariableID    int64  `json:"variable_id" binding:"required,min=1"`
	Key           string `json:"key" binding:"required,min=1,max=200"`
	Value         string `json:"value" binding:"max=2000"`
	Description   string `json:"description" binding:"max=1024"`
}

type DeleteRequest struct {
	WorkspaceID   int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID     int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	EnvironmentID int64 `form:"environment_id" json:"environment_id" binding:"required,min=1"`
	VariableID    int64 `form:"variable_id" json:"variable_id" binding:"required,min=1"`
}
