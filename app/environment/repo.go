package environment

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entenv "nest-api/internal/ent/environment"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

func EnsureExists(ctx context.Context, projectID, environmentID int64) error {
	return (Repo{}).EnsureExists(ctx, projectID, environmentID)
}

func (Repo) EnsureExists(ctx context.Context, projectID, environmentID int64) error {
	exists, err := database.DB.Environment.
		Query().
		Where(
			entenv.IDEQ(environmentID),
			entenv.ProjectIDEQ(projectID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("environment not found")
	}
	return nil
}

func (Repo) ListByProject(ctx context.Context, projectID int64) ([]*ent.Environment, error) {
	return database.DB.Environment.
		Query().
		Where(entenv.ProjectIDEQ(projectID)).
		Order(ent.Desc(entenv.FieldIsDefault), ent.Asc(entenv.FieldID)).
		All(ctx)
}

func (Repo) CountByProject(ctx context.Context, projectID int64) (int, error) {
	return database.DB.Environment.
		Query().
		Where(entenv.ProjectIDEQ(projectID)).
		Count(ctx)
}

func (Repo) ClearDefault(ctx context.Context, projectID int64) error {
	_, err := database.DB.Environment.
		Update().
		Where(
			entenv.ProjectIDEQ(projectID),
			entenv.IsDefaultEQ(true),
		).
		SetIsDefault(false).
		Save(ctx)
	return err
}

func (Repo) Create(ctx context.Context, projectID, userID int64, name, remark string, isDefault bool) (*ent.Environment, error) {
	return database.DB.Environment.
		Create().
		SetProjectID(projectID).
		SetName(name).
		SetRemark(remark).
		SetIsDefault(isDefault).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) Update(ctx context.Context, environmentID int64, name, remark string, isDefault bool) error {
	_, err := database.DB.Environment.
		UpdateOneID(environmentID).
		SetName(name).
		SetRemark(remark).
		SetIsDefault(isDefault).
		Save(ctx)
	return err
}

func (Repo) Delete(ctx context.Context, projectID, environmentID int64) (int, error) {
	return database.DB.Environment.
		Delete().
		Where(
			entenv.IDEQ(environmentID),
			entenv.ProjectIDEQ(projectID),
		).
		Exec(ctx)
}
