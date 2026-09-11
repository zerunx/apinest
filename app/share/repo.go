package share

import (
	"context"
	"strings"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entapi "nest-api/internal/ent/api"
	entfolder "nest-api/internal/ent/folder"
	"nest-api/internal/ent/projectshare"
	"nest-api/internal/ent/projectshareinterface"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

func (Repo) ListByProject(ctx context.Context, workspaceID, projectID int64) ([]*ent.ProjectShare, error) {
	return database.DB.ProjectShare.
		Query().
		Where(
			projectshare.WorkspaceIDEQ(workspaceID),
			projectshare.ProjectIDEQ(projectID),
		).
		WithItems().
		Order(ent.Desc(projectshare.FieldID)).
		All(ctx)
}

func (Repo) GetByID(ctx context.Context, workspaceID, shareID int64) (*ent.ProjectShare, error) {
	row, err := database.DB.ProjectShare.
		Query().
		Where(
			projectshare.IDEQ(shareID),
			projectshare.WorkspaceIDEQ(workspaceID),
		).
		WithItems().
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("share not found")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) GetByIDForUpdate(ctx context.Context, workspaceID, shareID int64) (*ent.ProjectShare, error) {
	row, err := database.DB.ProjectShare.
		Query().
		Where(
			projectshare.IDEQ(shareID),
			projectshare.WorkspaceIDEQ(workspaceID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("share not found")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) FindByCode(ctx context.Context, shareCode string) (*ent.ProjectShare, error) {
	row, err := database.DB.ProjectShare.
		Query().
		Where(projectshare.ShareCodeEQ(strings.TrimSpace(shareCode))).
		WithProject().
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("share link is invalid or closed")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) FindByCodeWithItems(ctx context.Context, shareCode string) (*ent.ProjectShare, error) {
	row, err := database.DB.ProjectShare.
		Query().
		Where(projectshare.ShareCodeEQ(strings.TrimSpace(shareCode))).
		WithProject().
		WithItems().
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("share link is invalid or closed")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) LoadEnabledShare(ctx context.Context, shareCode, password string) (*ent.ProjectShare, error) {
	row, err := (Repo{}).FindByCodeWithItems(ctx, shareCode)
	if err != nil {
		return nil, err
	}
	if !row.Enabled {
		return nil, bizerr.New("share is closed")
	}
	if strings.TrimSpace(row.Password) != "" {
		if !utils.Verify(password, row.Password) {
			return nil, bizerr.New("incorrect access password")
		}
	}
	return row, nil
}

func (Repo) ShareCodeExists(ctx context.Context, code string) (bool, error) {
	return database.DB.ProjectShare.
		Query().
		Where(projectshare.ShareCodeEQ(code)).
		Exist(ctx)
}

func (Repo) Create(ctx context.Context, tx *ent.Tx, projectID, workspaceID, userID int64, name, code string, enabled bool, password string, permission uint8) (*ent.ProjectShare, error) {
	return tx.ProjectShare.
		Create().
		SetProjectID(projectID).
		SetWorkspaceID(workspaceID).
		SetName(name).
		SetShareCode(code).
		SetEnabled(enabled).
		SetPassword(password).
		SetPermission(permission).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) Update(ctx context.Context, tx *ent.Tx, shareID int64, name string, enabled bool, password string, permission uint8) (*ent.ProjectShare, error) {
	return tx.ProjectShare.
		UpdateOneID(shareID).
		SetName(name).
		SetEnabled(enabled).
		SetPassword(password).
		SetPermission(permission).
		Save(ctx)
}

func (Repo) Delete(ctx context.Context, workspaceID, shareID int64) (int, error) {
	return database.DB.ProjectShare.
		Delete().
		Where(
			projectshare.IDEQ(shareID),
			projectshare.WorkspaceIDEQ(workspaceID),
		).
		Exec(ctx)
}

func (Repo) ReplaceInterfaces(ctx context.Context, tx *ent.Tx, shareID int64, interfaceIDs []int64) error {
	_, err := tx.ProjectShareInterface.
		Delete().
		Where(projectshareinterface.ShareIDEQ(shareID)).
		Exec(ctx)
	if err != nil {
		return err
	}

	builders := make([]*ent.ProjectShareInterfaceCreate, 0, len(interfaceIDs))
	for _, interfaceID := range interfaceIDs {
		builders = append(builders, tx.ProjectShareInterface.
			Create().
			SetShareID(shareID).
			SetInterfaceID(interfaceID),
		)
	}
	return tx.ProjectShareInterface.CreateBulk(builders...).Exec(ctx)
}

func (Repo) ListAPIsByIDs(ctx context.Context, projectID int64, interfaceIDs []int64) ([]*ent.API, error) {
	if len(interfaceIDs) == 0 {
		return []*ent.API{}, nil
	}
	return database.DB.API.
		Query().
		Where(
			entapi.ProjectIDEQ(projectID),
			entapi.IDIn(interfaceIDs...),
		).
		WithFolder().
		Order(ent.Asc(entapi.FieldSortOrder), ent.Asc(entapi.FieldID)).
		All(ctx)
}

func (Repo) ListFoldersByProject(ctx context.Context, projectID int64) ([]*ent.Folder, error) {
	return database.DB.Folder.
		Query().
		Where(entfolder.ProjectIDEQ(projectID)).
		Order(ent.Asc(entfolder.FieldSortOrder), ent.Asc(entfolder.FieldID)).
		All(ctx)
}
