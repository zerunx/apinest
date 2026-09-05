package admin

type UserItem struct {
	ID       int64  `json:"id"`
	Name     string `json:"name"`
	Account  string `json:"account"`
	Email    string `json:"email"`
	Avatar   string `json:"avatar"`
	IsAdmin  bool   `json:"is_admin"`
	Status   int8   `json:"status"`
}

type WorkspaceItem struct {
	ID        int64  `json:"id"`
	Name      string `json:"name"`
	OwnerID   int64  `json:"owner_id"`
	OwnerName string `json:"owner_name"`
	CreatedAt string `json:"created_at"`
}

type TransferWorkspaceRequest struct {
	OwnerID int64 `json:"owner_id" binding:"required,min=1"`
}

type UpdateUserRequest struct {
	Name     string `json:"name" binding:"omitempty,max=50"`
	Account  string `json:"account" binding:"required,min=3,max=50"`
	Email    string `json:"email" binding:"omitempty,max=100"`
	Password string `json:"password" binding:"omitempty,min=6,max=72"`
	IsAdmin  bool   `json:"is_admin"`
	Status   int8   `json:"status" binding:"oneof=0 1"`
}
