package workspace

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/user"
	"nest-api/internal/ent/workspace"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"

	"entgo.io/ent/dialect/sql"
)

type Service struct{}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) error {
	ws, err := database.DB.Workspace.
		Create().
		SetName(params.Name).
		SetOwnerID(userID).
		Save(ctx)
	if err != nil {
		return err
	}

	_, err = database.DB.WorkspaceMember.
		Create().
		SetWorkspaceID(ws.ID).
		SetUserID(userID).
		SetRole(RoleOwner).
		Save(ctx)
	if err != nil {
		return err
	}

	return nil
}

func (Service) List(ctx context.Context, userID int64) ([]Item, error) {
	members, err := database.DB.WorkspaceMember.
		Query().
		Where(workspacemember.UserIDEQ(userID)).
		WithWorkspace(func(q *ent.WorkspaceQuery) {
			q.WithOwner()
		}).
		Order(workspacemember.ByWorkspaceID(sql.OrderDesc())).
		All(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(members))
	for _, m := range members {
		ws := m.Edges.Workspace
		if ws == nil {
			continue
		}
		items = append(items, *toWorkspaceItem(ws, m.Role))
	}
	return items, nil
}

func (Service) Get(ctx context.Context, userID int64, params GetRequest) (*Item, error) {
	access, err := RequireMember(ctx, userID, params.WorkspaceID)
	if err != nil {
		return nil, err
	}

	ws, err := database.DB.Workspace.
		Query().
		Where(workspace.IDEQ(params.WorkspaceID)).
		WithOwner().
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("workspace not found")
		}
		return nil, err
	}

	return toWorkspaceItem(ws, access.Role), nil
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if _, err := Require(ctx, userID, params.WorkspaceID, ActionWorkspaceUpdate); err != nil {
		return err
	}

	_, err := database.DB.Workspace.
		UpdateOneID(params.WorkspaceID).
		SetName(params.Name).
		Save(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return bizerr.New("workspace not found")
		}
		return err
	}
	return nil
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if _, err := Require(ctx, userID, params.WorkspaceID, ActionWorkspaceDelete); err != nil {
		return err
	}

	return database.DB.Workspace.DeleteOneID(params.WorkspaceID).Exec(ctx)
}

func (Service) TransferOwner(ctx context.Context, userID int64, params TransferOwnerRequest) error {
	if _, err := Require(ctx, userID, params.WorkspaceID, ActionWorkspaceTransfer); err != nil {
		return err
	}
	if params.OwnerID == userID {
		return bizerr.New("cannot transfer to yourself")
	}

	exists, err := database.DB.User.
		Query().
		Where(user.IDEQ(params.OwnerID)).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("target user not found")
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return err
	}

	_, err = tx.Workspace.
		UpdateOneID(params.WorkspaceID).
		SetOwnerID(params.OwnerID).
		Save(ctx)
	if err != nil {
		_ = tx.Rollback()
		if ent.IsNotFound(err) {
			return bizerr.New("workspace not found")
		}
		return err
	}

	if err = setMemberRole(ctx, tx, params.WorkspaceID, userID, RoleAdmin); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err = ensureMemberRole(ctx, tx, params.WorkspaceID, params.OwnerID, RoleOwner); err != nil {
		_ = tx.Rollback()
		return err
	}

	return tx.Commit()
}

func setMemberRole(ctx context.Context, tx *ent.Tx, workspaceID, userID int64, role uint8) error {
	member, err := tx.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(workspaceID),
			workspacemember.UserIDEQ(userID),
		).
		Only(ctx)
	if err != nil {
		return err
	}
	_, err = tx.WorkspaceMember.UpdateOneID(member.ID).SetRole(role).Save(ctx)
	return err
}

func ensureMemberRole(ctx context.Context, tx *ent.Tx, workspaceID, userID int64, role uint8) error {
	exists, err := tx.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(workspaceID),
			workspacemember.UserIDEQ(userID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if exists {
		return setMemberRole(ctx, tx, workspaceID, userID, role)
	}
	_, err = tx.WorkspaceMember.
		Create().
		SetWorkspaceID(workspaceID).
		SetUserID(userID).
		SetRole(role).
		Save(ctx)
	return err
}

func toWorkspaceItem(ws *ent.Workspace, role uint8) *Item {
	return &Item{
		ID:        ws.ID,
		Name:      ws.Name,
		OwnerID:   ws.OwnerID,
		OwnerName: UserDisplayName(ws.Edges.Owner),
		Role:      role,
		CreatedAt: ws.CreatedAt.Format(utils.DateTimeFormat),
	}
}
