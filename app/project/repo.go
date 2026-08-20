package project

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entproject "nest-api/internal/ent/project"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

// EnsureExists 校验项目是否存在于指定工作空间。
func EnsureExists(ctx context.Context, workspaceID, projectID int64) error {
	return (Repo{}).EnsureExists(ctx, workspaceID, projectID)
}

func (Repo) EnsureExists(ctx context.Context, workspaceID, projectID int64) error {
	exists, err := database.DB.Project.
		Query().
		Where(
			entproject.IDEQ(projectID),
			entproject.WorkspaceIDEQ(workspaceID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("项目不存在")
	}
	return nil
}

func (Repo) Get(ctx context.Context, workspaceID, projectID int64) (*ent.Project, error) {
	row, err := database.DB.Project.
		Query().
		Where(
			entproject.IDEQ(projectID),
			entproject.WorkspaceIDEQ(workspaceID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("项目不存在")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) ListByWorkspace(ctx context.Context, workspaceID int64) ([]*ent.Project, error) {
	return database.DB.Project.
		Query().
		Where(entproject.WorkspaceIDEQ(workspaceID)).
		WithCreator().
		Order(ent.Desc(entproject.FieldID)).
		All(ctx)
}

func (Repo) Create(ctx context.Context, workspaceID, userID int64, name string) (*ent.Project, error) {
	return database.DB.Project.
		Create().
		SetWorkspaceID(workspaceID).
		SetName(name).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) UpdateName(ctx context.Context, projectID int64, name string) error {
	_, err := database.DB.Project.
		UpdateOneID(projectID).
		SetName(name).
		Save(ctx)
	return err
}

func (Repo) Delete(ctx context.Context, workspaceID, projectID int64) (int, error) {
	return database.DB.Project.
		Delete().
		Where(
			entproject.IDEQ(projectID),
			entproject.WorkspaceIDEQ(workspaceID),
		).
		Exec(ctx)
}
