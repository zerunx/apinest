package environment

import (
	"context"

	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Service struct{}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}

	rows, err := Repo{}.ListByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(rows))
	for _, row := range rows {
		items = append(items, Item{
			ID:        row.ID,
			ProjectID: row.ProjectID,
			Name:      row.Name,
			Remark:    row.Remark,
			IsDefault: row.IsDefault,
			CreatedAt: row.CreatedAt.Format(utils.DateTimeFormat),
		})
	}
	return items, nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectCreate); err != nil {
		return err
	}

	isDefault := params.IsDefault
	count, err := Repo{}.CountByProject(ctx, params.ProjectID)
	if err != nil {
		return err
	}
	if count == 0 {
		isDefault = true
	}

	if isDefault {
		if err := (Repo{}).ClearDefault(ctx, params.ProjectID); err != nil {
			return err
		}
	}

	_, err = Repo{}.Create(ctx, params.ProjectID, userID, params.Name, params.Remark, isDefault)
	return err
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.EnvironmentID); err != nil {
		return err
	}

	if params.IsDefault {
		if err := (Repo{}).ClearDefault(ctx, params.ProjectID); err != nil {
			return err
		}
	}

	return (Repo{}).Update(ctx, params.EnvironmentID, params.Name, params.Remark, params.IsDefault)
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectDelete); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.EnvironmentID); err != nil {
		return err
	}

	n, err := Repo{}.Delete(ctx, params.ProjectID, params.EnvironmentID)
	if err != nil {
		return err
	}
	if n == 0 {
		return bizerr.New("environment not found")
	}
	return nil
}
