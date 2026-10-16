package workspace

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entworkspace "nest-api/internal/ent/workspace"
	"nest-api/internal/ent/workspacemember"
	bizerr "nest-api/pkg/errors"
)

type Access struct {
	WorkspaceID int64
	UserID      int64
	Role        uint8
	Member      *ent.WorkspaceMember
}

func RequireMember(ctx context.Context, userID, workspaceID int64) (*Access, error) {
	// Soft-deleted workspaces must not be accessible even if membership rows remain.
	exists, err := database.DB.Workspace.
		Query().
		Where(entworkspace.IDEQ(workspaceID)).
		Exist(ctx)
	if err != nil {
		return nil, err
	}
	if !exists {
		return nil, bizerr.New("workspace not found")
	}

	member, err := getMembership(ctx, workspaceID, userID)
	if err != nil {
		return nil, err
	}
	return &Access{
		WorkspaceID: workspaceID,
		UserID:      userID,
		Role:        member.Role,
		Member:      member,
	}, nil
}

func Require(ctx context.Context, userID, workspaceID int64, action Action) (*Access, error) {
	access, err := RequireMember(ctx, userID, workspaceID)
	if err != nil {
		return nil, err
	}
	if !action.Allowed(access.Role) {
		return nil, bizerr.New("permission denied")
	}
	return access, nil
}

func getMembership(ctx context.Context, workspaceID, userID int64) (*ent.WorkspaceMember, error) {
	member, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(workspaceID),
			workspacemember.UserIDEQ(userID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("no permission to access this workspace")
		}
		return nil, err
	}
	return member, nil
}
