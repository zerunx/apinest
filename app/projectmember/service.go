package projectmember

import (
	"context"

	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/projectmember"
	"nest-api/internal/ent/user"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Item struct {
	ID        int64  `json:"id"`
	UserID    int64  `json:"user_id"`
	Name      string `json:"name"`
	Account   string `json:"account"`
	Avatar    string `json:"avatar"`
	CreatedAt string `json:"created_at"`
}

type ListRequest struct {
	WorkspaceID int64 `form:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" binding:"required,min=1"`
}

type AddRequest struct {
	WorkspaceID int64 `json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `json:"project_id" binding:"required,min=1"`
	UserID      int64 `json:"user_id" binding:"required,min=1"`
}

type RemoveRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	UserID      int64 `form:"user_id" json:"user_id" binding:"required,min=1"`
}

type Service struct{}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionMemberInvite); err != nil {
		return nil, err
	}
	if err := project.EnsureExists(ctx, params.WorkspaceID, params.ProjectID); err != nil {
		return nil, err
	}

	rows, err := database.DB.ProjectMember.
		Query().
		Where(
			projectmember.WorkspaceIDEQ(params.WorkspaceID),
			projectmember.ProjectIDEQ(params.ProjectID),
			projectmember.HasUser(),
		).
		WithUser().
		Order(ent.Asc(projectmember.FieldID)).
		All(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(rows))
	for _, row := range rows {
		u := row.Edges.User
		if u == nil {
			continue
		}
		items = append(items, Item{
			ID:        row.ID,
			UserID:    row.UserID,
			CreatedAt: row.CreatedAt.Format(utils.DateTimeFormat),
			Name:      u.Name,
			Account:   u.Account,
			Avatar:    u.Avatar,
		})
	}
	return items, nil
}

func (Service) Add(ctx context.Context, operatorID int64, params AddRequest) error {
	if _, err := workspace.Require(ctx, operatorID, params.WorkspaceID, workspace.ActionMemberInvite); err != nil {
		return err
	}
	if err := project.EnsureExists(ctx, params.WorkspaceID, params.ProjectID); err != nil {
		return err
	}

	exists, err := database.DB.User.Query().Where(user.IDEQ(params.UserID)).Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("用户不存在")
	}

	isWSMember, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(params.WorkspaceID),
			workspacemember.UserIDEQ(params.UserID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !isWSMember {
		return bizerr.New("只能添加已加入该工作空间的成员")
	}

	return project.AddMember(ctx, params.WorkspaceID, params.ProjectID, params.UserID)
}

func (Service) Remove(ctx context.Context, operatorID int64, params RemoveRequest) error {
	if _, err := workspace.Require(ctx, operatorID, params.WorkspaceID, workspace.ActionMemberInvite); err != nil {
		return err
	}
	if err := project.EnsureExists(ctx, params.WorkspaceID, params.ProjectID); err != nil {
		return err
	}

	p, err := project.Repo{}.Get(ctx, params.WorkspaceID, params.ProjectID)
	if err != nil {
		return err
	}
	if p.CreatedBy == params.UserID {
		return bizerr.New("不能移除项目创建者")
	}

	return project.RemoveMember(ctx, params.ProjectID, params.UserID)
}
