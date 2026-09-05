package member

type Item struct {
	ID        int64  `json:"id"`
	UserID    int64  `json:"user_id"`
	Name      string `json:"name"`
	Account   string `json:"account"`
	Avatar    string `json:"avatar"`
	Role      uint8  `json:"role"`
	CreatedAt string `json:"created_at"`
}

type ListRequest struct {
	WorkspaceID int64 `form:"workspace_id" binding:"required,min=1"`
}

type InviteRequest struct {
	WorkspaceID int64 `json:"workspace_id" binding:"required,min=1"`
	UserID      int64 `json:"user_id" binding:"required,min=1"`
	Role        uint8 `json:"role" binding:"required,oneof=2 3 4"`
}

type UpdateRequest struct {
	WorkspaceID int64 `json:"workspace_id" binding:"required,min=1"`
	MemberID    int64 `json:"member_id" binding:"required,min=1"`
	Role        uint8 `json:"role" binding:"required,oneof=2 3 4"`
}

type DeleteRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	MemberID    int64 `form:"member_id" json:"member_id" binding:"required,min=1"`
}

type InviteLinkRequest struct {
	WorkspaceID int64  `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	Role        uint8  `form:"role" json:"role"`
}

type InviteLinkResponse struct {
	WorkspaceID   int64  `json:"workspace_id"`
	WorkspaceName string `json:"workspace_name"`
	InviteCode    string `json:"invite_code"`
	InviteURL     string `json:"invite_url"`
	SiteURL       string `json:"site_url"`
	Role          uint8  `json:"role"`
	ExpiresAt     string `json:"expires_at"`
}

type InvitePreviewRequest struct {
	InviteCode string `form:"invite_code" binding:"required,min=1,max=32"`
}

type InvitePreviewResponse struct {
	WorkspaceID   int64  `json:"workspace_id"`
	WorkspaceName string `json:"workspace_name"`
	InviteCode    string `json:"invite_code"`
	Role          uint8  `json:"role"`
	ExpiresAt     string `json:"expires_at"`
}

type AcceptInviteRequest struct {
	InviteCode string `json:"invite_code" binding:"required,min=1,max=32"`
}

type AcceptInviteResponse struct {
	WorkspaceID   int64  `json:"workspace_id"`
	WorkspaceName string `json:"workspace_name"`
	AlreadyMember bool   `json:"already_member"`
	Role          uint8  `json:"role"`
}
