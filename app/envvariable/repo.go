package envvariable

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entenvvar "nest-api/internal/ent/environmentvariable"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

func EnsureExists(ctx context.Context, environmentID, variableID int64) error {
	return (Repo{}).EnsureExists(ctx, environmentID, variableID)
}

func (Repo) EnsureExists(ctx context.Context, environmentID, variableID int64) error {
	exists, err := database.DB.EnvironmentVariable.
		Query().
		Where(
			entenvvar.IDEQ(variableID),
			entenvvar.EnvironmentIDEQ(environmentID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("variable not found")
	}
	return nil
}

func (Repo) ListByEnvironment(ctx context.Context, environmentID int64) ([]*ent.EnvironmentVariable, error) {
	return database.DB.EnvironmentVariable.
		Query().
		Where(entenvvar.EnvironmentIDEQ(environmentID)).
		Order(ent.Asc(entenvvar.FieldKey)).
		All(ctx)
}

func (Repo) KeyExists(ctx context.Context, environmentID int64, key string, excludeID int64) (bool, error) {
	q := database.DB.EnvironmentVariable.
		Query().
		Where(
			entenvvar.EnvironmentIDEQ(environmentID),
			entenvvar.KeyEQ(key),
		)
	if excludeID > 0 {
		q = q.Where(entenvvar.IDNEQ(excludeID))
	}
	return q.Exist(ctx)
}

func (Repo) Create(ctx context.Context, environmentID, userID int64, key, value, description string) (*ent.EnvironmentVariable, error) {
	return database.DB.EnvironmentVariable.
		Create().
		SetEnvironmentID(environmentID).
		SetKey(key).
		SetValue(value).
		SetDescription(description).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) Update(ctx context.Context, variableID int64, key, value, description string) error {
	_, err := database.DB.EnvironmentVariable.
		UpdateOneID(variableID).
		SetKey(key).
		SetValue(value).
		SetDescription(description).
		Save(ctx)
	return err
}

func (Repo) Delete(ctx context.Context, environmentID, variableID int64) (int, error) {
	return database.DB.EnvironmentVariable.
		Delete().
		Where(
			entenvvar.IDEQ(variableID),
			entenvvar.EnvironmentIDEQ(environmentID),
		).
		Exec(ctx)
}
