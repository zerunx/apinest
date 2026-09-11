package admin

import (
	"context"
	"regexp"
	"strings"

	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/projectmember"
	"nest-api/internal/ent/user"
	entworkspace "nest-api/internal/ent/workspace"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

var accountPattern = regexp.MustCompile(`^[a-zA-Z0-9_]+$`)

type Service struct{}

func (Service) ListUsers(ctx context.Context) ([]UserItem, error) {
	users, err := database.DB.User.
		Query().
		Order(ent.Desc(user.FieldID)).
		All(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]UserItem, 0, len(users))
	for _, u := range users {
		items = append(items, UserItem{
			ID:      u.ID,
			Name:    u.Name,
			Account: u.Account,
			Email:   u.Email,
			Avatar:  u.Avatar,
			IsAdmin: u.IsAdmin,
			Status:  u.Status,
		})
	}

	return items, nil
}

func (Service) ListWorkspaces(ctx context.Context) ([]WorkspaceItem, error) {
	workspaces, err := database.DB.Workspace.
		Query().
		WithOwner().
		Order(ent.Desc(entworkspace.FieldID)).
		All(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]WorkspaceItem, 0, len(workspaces))
	for _, ws := range workspaces {
		ownerName := ""
		if ws.Edges.Owner != nil {
			ownerName = ws.Edges.Owner.Name
			if ownerName == "" {
				ownerName = ws.Edges.Owner.Account
			}
		}

		items = append(items, WorkspaceItem{
			ID:        ws.ID,
			Name:      ws.Name,
			OwnerID:   ws.OwnerID,
			OwnerName: ownerName,
			CreatedAt: ws.CreatedAt.Format(utils.DateTimeFormat),
		})
	}

	return items, nil
}

func (Service) TransferWorkspace(ctx context.Context, workspaceID, ownerID int64) error {
	exists, err := database.DB.User.
		Query().
		Where(user.IDEQ(ownerID)).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("target user not found")
	}

	ws, err := database.DB.Workspace.Get(ctx, workspaceID)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("workspace not found")
		}
		return err
	}
	if ws.OwnerID == ownerID {
		return nil
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return err
	}

	_, err = tx.Workspace.
		UpdateOneID(workspaceID).
		SetOwnerID(ownerID).
		Save(ctx)
	if err != nil {
		_ = tx.Rollback()
		return err
	}

	if err = syncOwnerMember(ctx, tx, workspaceID, ws.OwnerID, ownerID); err != nil {
		_ = tx.Rollback()
		return err
	}

	return tx.Commit()
}

func (Service) UpdateUser(ctx context.Context, operatorID, targetID int64, params UpdateUserRequest) error {
	target, err := database.DB.User.
		Query().
		Where(user.IDEQ(targetID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("user not found")
		}
		return err
	}

	account := strings.TrimSpace(params.Account)
	if !accountPattern.MatchString(account) {
		return bizerr.New("account may only contain letters, digits, and underscores")
	}

	name := strings.TrimSpace(params.Name)
	email := strings.TrimSpace(strings.ToLower(params.Email))
	status := params.Status

	if account != target.Account {
		exists, err := database.DB.User.
			Query().
			Where(user.AccountEQ(account), user.IDNEQ(targetID)).
			Exist(ctx)
		if err != nil {
			return err
		}
		if exists {
			return bizerr.New("account already exists")
		}
	}

	if email != "" && email != target.Email {
		exists, err := database.DB.User.
			Query().
			Where(user.EmailEQ(email), user.IDNEQ(targetID)).
			Exist(ctx)
		if err != nil {
			return err
		}
		if exists {
			return bizerr.New("email is already in use")
		}
	}

	if target.IsAdmin && !params.IsAdmin {
		if operatorID == targetID {
			return bizerr.New("cannot remove your own admin role")
		}
		count, err := database.DB.User.
			Query().
			Where(user.IsAdminEQ(true), user.IDNEQ(targetID)).
			Count(ctx)
		if err != nil {
			return err
		}
		if count == 0 {
			return bizerr.New("at least one admin must remain")
		}
	}

	if target.Status == 1 && status == 0 && operatorID == targetID {
		return bizerr.New("cannot disable your own account")
	}

	upd := database.DB.User.
		UpdateOneID(targetID).
		SetName(name).
		SetAccount(account).
		SetEmail(email).
		SetIsAdmin(params.IsAdmin).
		SetStatus(status)

	if pwd := strings.TrimSpace(params.Password); pwd != "" {
		hash, err := utils.Hash(pwd)
		if err != nil {
			return err
		}
		upd = upd.SetPassword(hash)
	}

	_, err = upd.Save(ctx)
	return err
}

func (Service) DeleteUser(ctx context.Context, operatorID, targetID int64) error {
	if operatorID == targetID {
		return bizerr.New("cannot delete your own account")
	}

	target, err := database.DB.User.
		Query().
		Where(user.IDEQ(targetID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("user not found")
		}
		return err
	}

	if target.IsAdmin {
		count, err := database.DB.User.
			Query().
			Where(user.IsAdminEQ(true), user.IDNEQ(targetID)).
			Count(ctx)
		if err != nil {
			return err
		}
		if count == 0 {
			return bizerr.New("at least one admin must remain")
		}
	}

	owned, err := database.DB.Workspace.
		Query().
		Where(entworkspace.OwnerIDEQ(targetID)).
		Exist(ctx)
	if err != nil {
		return err
	}
	if owned {
		return bizerr.New("user still owns workspaces; transfer them before deleting")
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return err
	}

	if _, err = tx.ProjectMember.Delete().Where(projectmember.UserIDEQ(targetID)).Exec(ctx); err != nil {
		_ = tx.Rollback()
		return err
	}
	if _, err = tx.WorkspaceMember.Delete().Where(workspacemember.UserIDEQ(targetID)).Exec(ctx); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err = tx.User.DeleteOneID(targetID).Exec(ctx); err != nil {
		_ = tx.Rollback()
		return err
	}
	return tx.Commit()
}

func syncOwnerMember(ctx context.Context, tx *ent.Tx, workspaceID, oldOwnerID, newOwnerID int64) error {
	oldMember, err := tx.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(workspaceID),
			workspacemember.UserIDEQ(oldOwnerID),
		).
		Only(ctx)
	if err == nil {
		if _, err = tx.WorkspaceMember.UpdateOneID(oldMember.ID).SetRole(workspace.RoleAdmin).Save(ctx); err != nil {
			return err
		}
	} else if !ent.IsNotFound(err) {
		return err
	}

	newMember, err := tx.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(workspaceID),
			workspacemember.UserIDEQ(newOwnerID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			_, err = tx.WorkspaceMember.
				Create().
				SetWorkspaceID(workspaceID).
				SetUserID(newOwnerID).
				SetRole(workspace.RoleOwner).
				Save(ctx)
			return err
		}
		return err
	}

	_, err = tx.WorkspaceMember.UpdateOneID(newMember.ID).SetRole(workspace.RoleOwner).Save(ctx)
	return err
}
