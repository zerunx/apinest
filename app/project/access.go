package project

import (
	"context"

	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent/projectmember"
	bizerr "nest-api/pkg/errors"
)

func RequireAccess(ctx context.Context, userID, workspaceID, projectID int64, action workspace.Action) (*workspace.Access, error) {
	access, err := workspace.Require(ctx, userID, workspaceID, action)
	if err != nil {
		return nil, err
	}
	if err := EnsureExists(ctx, workspaceID, projectID); err != nil {
		return nil, err
	}
	if workspace.IsWorkspaceAdmin(access.Role) {
		return access, nil
	}
	ok, err := IsMember(ctx, projectID, userID)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, bizerr.New("no permission to access this project")
	}
	return access, nil
}

func IsMember(ctx context.Context, projectID, userID int64) (bool, error) {
	return database.DB.ProjectMember.
		Query().
		Where(
			projectmember.ProjectIDEQ(projectID),
			projectmember.UserIDEQ(userID),
		).
		Exist(ctx)
}

func AddMember(ctx context.Context, workspaceID, projectID, userID int64) error {
	ok, err := IsMember(ctx, projectID, userID)
	if err != nil {
		return err
	}
	if ok {
		return nil
	}
	_, err = database.DB.ProjectMember.
		Create().
		SetWorkspaceID(workspaceID).
		SetProjectID(projectID).
		SetUserID(userID).
		Save(ctx)
	return err
}

func RemoveMember(ctx context.Context, projectID, userID int64) error {
	_, err := database.DB.ProjectMember.
		Delete().
		Where(
			projectmember.ProjectIDEQ(projectID),
			projectmember.UserIDEQ(userID),
		).
		Exec(ctx)
	return err
}

func ListMemberUserIDs(ctx context.Context, projectID int64) ([]int64, error) {
	rows, err := database.DB.ProjectMember.
		Query().
		Where(projectmember.ProjectIDEQ(projectID)).
		All(ctx)
	if err != nil {
		return nil, err
	}
	ids := make([]int64, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.UserID)
	}
	return ids, nil
}

func ListProjectIDsForUser(ctx context.Context, workspaceID, userID int64) ([]int64, error) {
	rows, err := database.DB.ProjectMember.
		Query().
		Where(
			projectmember.WorkspaceIDEQ(workspaceID),
			projectmember.UserIDEQ(userID),
		).
		All(ctx)
	if err != nil {
		return nil, err
	}
	ids := make([]int64, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.ProjectID)
	}
	return ids, nil
}
