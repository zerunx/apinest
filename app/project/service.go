package project

import (
	"context"

	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	entproject "nest-api/internal/ent/project"
	"nest-api/internal/ent/projectmember"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Service struct{}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	access, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionProjectRead)
	if err != nil {
		return nil, err
	}

	var projects []*ent.Project
	if workspace.IsWorkspaceAdmin(access.Role) {
		projects, err = Repo{}.ListByWorkspace(ctx, params.WorkspaceID)
	} else {
		ids, listErr := ListProjectIDsForUser(ctx, params.WorkspaceID, userID)
		if listErr != nil {
			return nil, listErr
		}
		if len(ids) == 0 {
			return []Item{}, nil
		}
		projects, err = database.DB.Project.
			Query().
			Where(
				entproject.WorkspaceIDEQ(params.WorkspaceID),
				entproject.IDIn(ids...),
			).
			WithCreator().
			Order(ent.Desc(entproject.FieldID)).
			All(ctx)
	}
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(projects))
	for _, p := range projects {
		items = append(items, Item{
			ID:          p.ID,
			WorkspaceID: p.WorkspaceID,
			Name:        p.Name,
			CreatedBy:   p.CreatedBy,
			CreatorName: workspace.UserDisplayName(p.Edges.Creator),
			CreatedAt:   p.CreatedAt.Format(utils.DateTimeFormat),
		})
	}
	return items, nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) (int64, error) {
	if _, err := workspace.Require(ctx, userID, params.WorkspaceID, workspace.ActionProjectCreate); err != nil {
		return 0, err
	}

	p, err := Repo{}.Create(ctx, params.WorkspaceID, userID, params.Name)
	if err != nil {
		return 0, err
	}
	if err := AddMember(ctx, params.WorkspaceID, p.ID, userID); err != nil {
		return 0, err
	}
	return p.ID, nil
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	access, err := RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate)
	if err != nil {
		return err
	}
	if err := assertCanManageProject(ctx, access, params.WorkspaceID, params.ProjectID); err != nil {
		return err
	}
	return (Repo{}).UpdateName(ctx, params.ProjectID, params.Name)
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	access, err := RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectDelete)
	if err != nil {
		return err
	}
	if err := assertCanManageProject(ctx, access, params.WorkspaceID, params.ProjectID); err != nil {
		return err
	}

	n, err := Repo{}.Delete(ctx, params.WorkspaceID, params.ProjectID)
	if err != nil {
		return err
	}
	if n == 0 {
		return bizerr.New("project not found")
	}
	_, err = database.DB.ProjectMember.
		Delete().
		Where(projectmember.ProjectIDEQ(params.ProjectID)).
		Exec(ctx)
	return err
}

func assertCanManageProject(ctx context.Context, access *workspace.Access, workspaceID, projectID int64) error {
	if workspace.IsWorkspaceAdmin(access.Role) {
		return nil
	}
	p, err := Repo{}.Get(ctx, workspaceID, projectID)
	if err != nil {
		return err
	}
	if p.CreatedBy != access.UserID {
		return bizerr.New("you can only modify or delete projects you created")
	}
	return nil
}

func BackfillMembersForExistingProjects(ctx context.Context) error {
	projects, err := database.DB.Project.Query().All(ctx)
	if err != nil {
		return err
	}
	for _, p := range projects {
		if err := AddMember(ctx, p.WorkspaceID, p.ID, p.CreatedBy); err != nil {
			return err
		}
		admins, err := database.DB.WorkspaceMember.
			Query().
			Where(
				workspacemember.WorkspaceIDEQ(p.WorkspaceID),
				workspacemember.RoleIn(workspace.RoleOwner, workspace.RoleAdmin),
			).
			All(ctx)
		if err != nil {
			return err
		}
		for _, m := range admins {
			if err := AddMember(ctx, p.WorkspaceID, p.ID, m.UserID); err != nil {
				return err
			}
		}
	}
	return nil
}
