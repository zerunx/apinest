package member

import (
	"context"

	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/user"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Service struct{}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionMemberRead); err != nil {
		return nil, err
	}

	members, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(params.WorkspaceID),
			workspacemember.HasUser(),
		).
		WithUser().
		Order(ent.Asc(workspacemember.FieldID)).
		All(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(members))
	for _, m := range members {
		u := m.Edges.User
		if u == nil {
			continue
		}
		items = append(items, Item{
			ID:        m.ID,
			UserID:    m.UserID,
			Role:      m.Role,
			CreatedAt: m.CreatedAt.Format(utils.DateTimeFormat),
			Name:      u.Name,
			Account:   u.Account,
			Avatar:    u.Avatar,
		})
	}
	return items, nil
}

func (Service) Invite(ctx context.Context, userID int64, params InviteRequest) error {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionMemberInvite); err != nil {
		return err
	}

	exists, err := database.DB.User.
		Query().
		Where(user.IDEQ(params.UserID)).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("用户不存在")
	}

	exists, err = database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(params.WorkspaceID),
			workspacemember.UserIDEQ(params.UserID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if exists {
		return bizerr.New("用户已是工作空间成员")
	}

	_, err = database.DB.WorkspaceMember.
		Create().
		SetWorkspaceID(params.WorkspaceID).
		SetUserID(params.UserID).
		SetRole(params.Role).
		Save(ctx)
	return err
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionMemberUpdateRole); err != nil {
		return err
	}

	target, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.IDEQ(params.MemberID),
			workspacemember.WorkspaceIDEQ(params.WorkspaceID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("成员不存在")
		}
		return err
	}
	if target.Role == workspace.RoleOwner {
		return bizerr.New("不能修改 Owner 角色")
	}

	_, err = database.DB.WorkspaceMember.
		UpdateOneID(params.MemberID).
		SetRole(params.Role).
		Save(ctx)
	return err
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionMemberRemove); err != nil {
		return err
	}

	target, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.IDEQ(params.MemberID),
			workspacemember.WorkspaceIDEQ(params.WorkspaceID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("成员不存在")
		}
		return err
	}
	if target.Role == workspace.RoleOwner {
		return bizerr.New("不能移除 Owner")
	}

	return database.DB.WorkspaceMember.DeleteOneID(params.MemberID).Exec(ctx)
}
