package folder

import (
	"context"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entapi "nest-api/internal/ent/api"
	entfolder "nest-api/internal/ent/folder"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

// EnsureExists 校验文件夹是否存在于指定项目。
func EnsureExists(ctx context.Context, projectID, folderID int64) error {
	return (Repo{}).EnsureExists(ctx, projectID, folderID)
}

func (Repo) EnsureExists(ctx context.Context, projectID, folderID int64) error {
	exists, err := database.DB.Folder.
		Query().
		Where(
			entfolder.IDEQ(folderID),
			entfolder.ProjectIDEQ(projectID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("文件夹不存在")
	}
	return nil
}

func (Repo) ListByProject(ctx context.Context, projectID int64) ([]*ent.Folder, error) {
	return database.DB.Folder.
		Query().
		Where(entfolder.ProjectIDEQ(projectID)).
		Order(ent.Asc(entfolder.FieldSortOrder), ent.Asc(entfolder.FieldID)).
		All(ctx)
}

func (Repo) ListAPIsByProject(ctx context.Context, projectID int64) ([]*ent.API, error) {
	return database.DB.API.
		Query().
		Where(entapi.ProjectIDEQ(projectID)).
		Order(ent.Asc(entapi.FieldSortOrder), ent.Asc(entapi.FieldID)).
		All(ctx)
}

func (Repo) Get(ctx context.Context, projectID, folderID int64) (*ent.Folder, error) {
	row, err := database.DB.Folder.
		Query().
		Where(
			entfolder.IDEQ(folderID),
			entfolder.ProjectIDEQ(projectID),
		).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("文件夹不存在")
		}
		return nil, err
	}
	return row, nil
}

func (Repo) NextSortOrder(ctx context.Context, projectID, parentID int64) (int, error) {
	return nextFolderSortOrder(ctx, database.DB.Folder.Query(), projectID, parentID)
}

func (Repo) NextSortOrderTx(ctx context.Context, tx *ent.Tx, projectID, parentID int64) (int, error) {
	return nextFolderSortOrder(ctx, tx.Folder.Query(), projectID, parentID)
}

func nextFolderSortOrder(ctx context.Context, q *ent.FolderQuery, projectID, parentID int64) (int, error) {
	rows, err := q.
		Where(
			entfolder.ProjectIDEQ(projectID),
			entfolder.ParentIDEQ(parentID),
		).
		Order(ent.Desc(entfolder.FieldSortOrder)).
		Limit(1).
		All(ctx)
	if err != nil {
		return 0, err
	}
	if len(rows) == 0 {
		return 0, nil
	}
	return rows[0].SortOrder + 1, nil
}

func (Repo) Create(ctx context.Context, projectID, parentID, userID int64, name string) (*ent.Folder, error) {
	return database.DB.Folder.
		Create().
		SetProjectID(projectID).
		SetParentID(parentID).
		SetName(name).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) CreateTx(ctx context.Context, tx *ent.Tx, projectID, parentID, userID int64, name string, sortOrder int) (*ent.Folder, error) {
	return tx.Folder.
		Create().
		SetProjectID(projectID).
		SetParentID(parentID).
		SetName(name).
		SetSortOrder(sortOrder).
		SetCreatedBy(userID).
		Save(ctx)
}

func (Repo) UpdateName(ctx context.Context, folderID int64, name string) error {
	_, err := database.DB.Folder.
		UpdateOneID(folderID).
		SetName(name).
		Save(ctx)
	return err
}

func (Repo) DeleteOne(ctx context.Context, folderID int64) error {
	return database.DB.Folder.DeleteOneID(folderID).Exec(ctx)
}

func (Repo) DeleteAPI(ctx context.Context, apiID int64) error {
	return database.DB.API.DeleteOneID(apiID).Exec(ctx)
}
