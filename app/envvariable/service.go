package envvariable

import (
	"context"

	"nest-api/app/environment"
	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Service struct{}

func requireScope(ctx context.Context, userID, workspaceID, projectID, environmentID int64, action workspace.Action) error {
	if _, err := project.RequireAccess(ctx, userID, workspaceID, projectID, action); err != nil {
		return err
	}
	return environment.EnsureExists(ctx, projectID, environmentID)
}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	if err := requireScope(ctx, userID, params.WorkspaceID, params.ProjectID, params.EnvironmentID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}

	rows, err := Repo{}.ListByEnvironment(ctx, params.EnvironmentID)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(rows))
	for _, row := range rows {
		items = append(items, Item{
			ID:            row.ID,
			EnvironmentID: row.EnvironmentID,
			Key:           row.Key,
			Value:         row.Value,
			Description:   row.Description,
			CreatedAt:     row.CreatedAt.Format(utils.DateTimeFormat),
			UpdatedAt:     row.UpdatedAt.Format(utils.DateTimeFormat),
		})
	}
	return items, nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) error {
	if err := requireScope(ctx, userID, params.WorkspaceID, params.ProjectID, params.EnvironmentID, workspace.ActionProjectCreate); err != nil {
		return err
	}

	exists, err := Repo{}.KeyExists(ctx, params.EnvironmentID, params.Key, 0)
	if err != nil {
		return err
	}
	if exists {
		return bizerr.New("变量名已存在")
	}

	_, err = Repo{}.Create(ctx, params.EnvironmentID, userID, params.Key, params.Value, params.Description)
	return err
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if err := requireScope(ctx, userID, params.WorkspaceID, params.ProjectID, params.EnvironmentID, workspace.ActionProjectUpdate); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.EnvironmentID, params.VariableID); err != nil {
		return err
	}

	exists, err := Repo{}.KeyExists(ctx, params.EnvironmentID, params.Key, params.VariableID)
	if err != nil {
		return err
	}
	if exists {
		return bizerr.New("变量名已存在")
	}

	return (Repo{}).Update(ctx, params.VariableID, params.Key, params.Value, params.Description)
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if err := requireScope(ctx, userID, params.WorkspaceID, params.ProjectID, params.EnvironmentID, workspace.ActionProjectDelete); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.EnvironmentID, params.VariableID); err != nil {
		return err
	}

	n, err := Repo{}.Delete(ctx, params.EnvironmentID, params.VariableID)
	if err != nil {
		return err
	}
	if n == 0 {
		return bizerr.New("变量不存在")
	}
	return nil
}
