package share

const (
	PermissionView uint8 = 1
)

type Item struct {
	ID             int64   `json:"id"`
	ProjectID      int64   `json:"project_id"`
	WorkspaceID    int64   `json:"workspace_id"`
	Name           string  `json:"name"`
	ShareCode      string  `json:"share_code"`
	ShareURL       string  `json:"share_url"`
	Enabled        bool    `json:"enabled"`
	HasPassword    bool    `json:"has_password"`
	Permission     uint8   `json:"permission"`
	InterfaceIDs   []int64 `json:"interface_ids,omitempty"`
	InterfaceCount int     `json:"interface_count"`
	CreatedAt      string  `json:"created_at"`
	UpdatedAt      string  `json:"updated_at"`
}

type ListRequest struct {
	WorkspaceID int64 `form:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" binding:"required,min=1"`
}

type GetRequest struct {
	WorkspaceID int64 `form:"workspace_id" binding:"required,min=1"`
	ShareID     int64 `form:"share_id" binding:"required,min=1"`
}

type CreateRequest struct {
	WorkspaceID  int64   `json:"workspace_id" binding:"required,min=1"`
	ProjectID    int64   `json:"project_id" binding:"required,min=1"`
	Name         string  `json:"name" binding:"required,min=1,max=50"`
	Enabled      *bool   `json:"enabled"`
	Password     string  `json:"password" binding:"omitempty,max=64"`
	Permission   uint8   `json:"permission" binding:"omitempty,oneof=1"`
	InterfaceIDs []int64 `json:"interface_ids"`
}

type UpdateRequest struct {
	WorkspaceID  int64   `json:"workspace_id" binding:"required,min=1"`
	ShareID      int64   `json:"share_id" binding:"required,min=1"`
	Name         string  `json:"name" binding:"required,min=1,max=50"`
	Enabled      *bool   `json:"enabled"`
	Password     *string `json:"password"`
	Permission   uint8   `json:"permission" binding:"omitempty,oneof=1"`
	InterfaceIDs []int64 `json:"interface_ids"`
}

type DeleteRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ShareID     int64 `form:"share_id" json:"share_id" binding:"required,min=1"`
}

type PreviewRequest struct {
	ShareCode string `form:"share_code" binding:"required,min=1,max=16"`
}

type PreviewResponse struct {
	ShareCode   string `json:"share_code"`
	Name        string `json:"name"`
	ProjectName string `json:"project_name"`
	Enabled     bool   `json:"enabled"`
	HasPassword bool   `json:"has_password"`
	Permission  uint8  `json:"permission"`
}

type AccessRequest struct {
	ShareCode string `json:"share_code" binding:"required,min=1,max=16"`
	Password  string `json:"password" binding:"omitempty,max=64"`
}

type AccessContentResponse struct {
	ShareCode   string            `json:"share_code"`
	Name        string            `json:"name"`
	ProjectName string            `json:"project_name"`
	Permission  uint8             `json:"permission"`
	Interfaces  []SharedInterface `json:"interfaces"`
	Folders     []SharedFolder    `json:"folders"`
	Tree        []ShareTreeNode   `json:"tree"`
}

type SharedFolder struct {
	ID       int64  `json:"id"`
	ParentID int64  `json:"parent_id"`
	Name     string `json:"name"`
}

type SharedInterface struct {
	ID         int64  `json:"id"`
	FolderID   int64  `json:"folder_id"`
	FolderName string `json:"folder_name,omitempty"`
	Name       string `json:"name"`
	Method     string `json:"method"`
	URL        string `json:"url"`
	Status     uint8  `json:"status"`
}

type ShareTreeNode struct {
	ID       int64           `json:"id"`
	Name     string          `json:"name"`
	Type     string          `json:"type"`
	Method   string          `json:"method,omitempty"`
	URL      string          `json:"url,omitempty"`
	Status   uint8           `json:"status,omitempty"`
	Children []ShareTreeNode `json:"children,omitempty"`
}

type AccessDetailRequest struct {
	ShareCode   string `form:"share_code" json:"share_code" binding:"required,min=1,max=16"`
	Password    string `form:"password" json:"password" binding:"omitempty,max=64"`
	InterfaceID int64  `form:"interface_id" json:"interface_id" binding:"required,min=1"`
}
