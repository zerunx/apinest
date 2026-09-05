package share

import (
	"context"
	"crypto/rand"
	"fmt"
	"math/big"
	"sort"
	"strings"
	"unicode/utf8"

	"nest-api/app/interfaces"
	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/runtime"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

const shareCodeCharset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
const shareCodeLength = 10
const shareNameMaxLen = 50

type Service struct{}

func generateShareCode() (string, error) {
	b := make([]byte, shareCodeLength)
	max := big.NewInt(int64(len(shareCodeCharset)))
	for i := range b {
		n, err := rand.Int(rand.Reader, max)
		if err != nil {
			return "", err
		}
		b[i] = shareCodeCharset[n.Int64()]
	}
	return string(b), nil
}

func buildShareURL(code string) string {
	base := strings.TrimRight(runtime.SiteURL(), "/")
	if base == "" {
		base = "http://localhost:5173"
	}
	return fmt.Sprintf("%s/share?shareCode=%s", base, code)
}

func uniqueInterfaceIDs(ids []int64) []int64 {
	seen := make(map[int64]struct{}, len(ids))
	out := make([]int64, 0, len(ids))
	for _, id := range ids {
		if id < 1 {
			continue
		}
		if _, ok := seen[id]; ok {
			continue
		}
		seen[id] = struct{}{}
		out = append(out, id)
	}
	return out
}

func normalizeShareName(name string) (string, error) {
	name = strings.TrimSpace(name)
	if name == "" {
		return "", bizerr.New("请输入分享名称")
	}
	if utf8.RuneCountInString(name) > shareNameMaxLen {
		return "", bizerr.New("分享名称不能超过 50 个字符")
	}
	return name, nil
}

func (Service) validateInterfaces(ctx context.Context, projectID int64, interfaceIDs []int64) ([]int64, error) {
	ids := uniqueInterfaceIDs(interfaceIDs)
	if len(ids) == 0 {
		return nil, bizerr.New("请至少选择一个接口")
	}

	count, err := (interfaces.Repo{}).CountInProject(ctx, projectID, ids)
	if err != nil {
		return nil, err
	}
	if count != len(ids) {
		return nil, bizerr.New("存在不属于该项目的接口")
	}
	return ids, nil
}

func toItem(row *ent.ProjectShare, interfaceIDs []int64) Item {
	count := len(interfaceIDs)
	if count == 0 && row.Edges.Items != nil {
		count = len(row.Edges.Items)
	}
	return Item{
		ID:             row.ID,
		ProjectID:      row.ProjectID,
		WorkspaceID:    row.WorkspaceID,
		Name:           row.Name,
		ShareCode:      row.ShareCode,
		ShareURL:       buildShareURL(row.ShareCode),
		Enabled:        row.Enabled,
		HasPassword:    strings.TrimSpace(row.Password) != "",
		Permission:     row.Permission,
		InterfaceIDs:   interfaceIDs,
		InterfaceCount: count,
		CreatedAt:      row.CreatedAt.Format(utils.DateTimeFormat),
		UpdatedAt:      row.UpdatedAt.Format(utils.DateTimeFormat),
	}
}

func (Service) List(ctx context.Context, userID int64, params ListRequest) ([]Item, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}

	rows, err := Repo{}.ListByProject(ctx, params.WorkspaceID, params.ProjectID)
	if err != nil {
		return nil, err
	}

	items := make([]Item, 0, len(rows))
	for _, row := range rows {
		ids := make([]int64, 0, len(row.Edges.Items))
		for _, item := range row.Edges.Items {
			ids = append(ids, item.InterfaceID)
		}
		items = append(items, toItem(row, ids))
	}
	return items, nil
}

func (Service) Get(ctx context.Context, userID int64, params GetRequest) (*Item, error) {
	row, err := Repo{}.GetByID(ctx, params.WorkspaceID, params.ShareID)
	if err != nil {
		return nil, err
	}
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, row.ProjectID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}

	ids := make([]int64, 0, len(row.Edges.Items))
	for _, item := range row.Edges.Items {
		ids = append(ids, item.InterfaceID)
	}
	result := toItem(row, ids)
	return &result, nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) (*Item, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return nil, err
	}

	interfaceIDs, err := (Service{}).validateInterfaces(ctx, params.ProjectID, params.InterfaceIDs)
	if err != nil {
		return nil, err
	}

	name, err := normalizeShareName(params.Name)
	if err != nil {
		return nil, err
	}

	permission := params.Permission
	if permission == 0 {
		permission = PermissionView
	}

	enabled := true
	if params.Enabled != nil {
		enabled = *params.Enabled
	}

	passwordHash := ""
	if strings.TrimSpace(params.Password) != "" {
		passwordHash, err = utils.Hash(params.Password)
		if err != nil {
			return nil, err
		}
	}

	repo := Repo{}
	var code string
	for range 8 {
		code, err = generateShareCode()
		if err != nil {
			return nil, err
		}
		exists, err := repo.ShareCodeExists(ctx, code)
		if err != nil {
			return nil, err
		}
		if !exists {
			break
		}
		code = ""
	}
	if code == "" {
		return nil, bizerr.New("生成分享码失败，请重试")
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return nil, err
	}

	row, err := repo.Create(ctx, tx, params.ProjectID, params.WorkspaceID, userID, name, code, enabled, passwordHash, permission)
	if err != nil {
		_ = tx.Rollback()
		return nil, err
	}

	if err = repo.ReplaceInterfaces(ctx, tx, row.ID, interfaceIDs); err != nil {
		_ = tx.Rollback()
		return nil, err
	}

	if err = tx.Commit(); err != nil {
		return nil, err
	}

	result := toItem(row, interfaceIDs)
	return &result, nil
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) (*Item, error) {
	repo := Repo{}
	row, err := repo.GetByIDForUpdate(ctx, params.WorkspaceID, params.ShareID)
	if err != nil {
		return nil, err
	}
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, row.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return nil, err
	}

	interfaceIDs, err := (Service{}).validateInterfaces(ctx, row.ProjectID, params.InterfaceIDs)
	if err != nil {
		return nil, err
	}

	name, err := normalizeShareName(params.Name)
	if err != nil {
		return nil, err
	}

	permission := params.Permission
	if permission == 0 {
		permission = PermissionView
	}

	enabled := row.Enabled
	if params.Enabled != nil {
		enabled = *params.Enabled
	}

	passwordHash := row.Password
	if params.Password != nil {
		raw := strings.TrimSpace(*params.Password)
		if raw == "" {
			passwordHash = ""
		} else {
			passwordHash, err = utils.Hash(raw)
			if err != nil {
				return nil, err
			}
		}
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return nil, err
	}

	updated, err := repo.Update(ctx, tx, row.ID, name, enabled, passwordHash, permission)
	if err != nil {
		_ = tx.Rollback()
		return nil, err
	}

	if err = repo.ReplaceInterfaces(ctx, tx, row.ID, interfaceIDs); err != nil {
		_ = tx.Rollback()
		return nil, err
	}

	if err = tx.Commit(); err != nil {
		return nil, err
	}

	result := toItem(updated, interfaceIDs)
	return &result, nil
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	row, err := Repo{}.GetByID(ctx, params.WorkspaceID, params.ShareID)
	if err != nil {
		return err
	}
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, row.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return err
	}

	n, err := Repo{}.Delete(ctx, params.WorkspaceID, params.ShareID)
	if err != nil {
		return err
	}
	if n == 0 {
		return bizerr.New("分享不存在")
	}
	return nil
}

func (Service) Preview(ctx context.Context, params PreviewRequest) (*PreviewResponse, error) {
	row, err := Repo{}.FindByCode(ctx, params.ShareCode)
	if err != nil {
		return nil, err
	}

	projectName := ""
	if row.Edges.Project != nil {
		projectName = row.Edges.Project.Name
	}

	return &PreviewResponse{
		ShareCode:   row.ShareCode,
		Name:        row.Name,
		ProjectName: projectName,
		Enabled:     row.Enabled,
		HasPassword: strings.TrimSpace(row.Password) != "",
		Permission:  row.Permission,
	}, nil
}

func (Service) AccessContent(ctx context.Context, params AccessRequest) (*AccessContentResponse, error) {
	repo := Repo{}
	row, err := repo.LoadEnabledShare(ctx, params.ShareCode, params.Password)
	if err != nil {
		return nil, err
	}

	interfaceIDs := make([]int64, 0, len(row.Edges.Items))
	for _, item := range row.Edges.Items {
		interfaceIDs = append(interfaceIDs, item.InterfaceID)
	}

	apis, err := repo.ListAPIsByIDs(ctx, row.ProjectID, interfaceIDs)
	if err != nil {
		return nil, err
	}

	folders, err := repo.ListFoldersByProject(ctx, row.ProjectID)
	if err != nil {
		return nil, err
	}

	list := make([]SharedInterface, 0, len(apis))
	for _, api := range apis {
		folderName := ""
		if api.Edges.Folder != nil {
			folderName = api.Edges.Folder.Name
		}
		list = append(list, SharedInterface{
			ID:         api.ID,
			FolderID:   api.FolderID,
			FolderName: folderName,
			Name:       api.Name,
			Method:     api.Method,
			URL:        api.URL,
			Status:     api.Status,
		})
	}

	folderItems := make([]SharedFolder, 0, len(folders))
	for _, folder := range folders {
		folderItems = append(folderItems, SharedFolder{
			ID:       folder.ID,
			ParentID: folder.ParentID,
			Name:     folder.Name,
		})
	}

	projectName := ""
	if row.Edges.Project != nil {
		projectName = row.Edges.Project.Name
	}

	return &AccessContentResponse{
		ShareCode:   row.ShareCode,
		Name:        row.Name,
		ProjectName: projectName,
		Permission:  row.Permission,
		Interfaces:  list,
		Folders:     folderItems,
		Tree:        buildShareTree(folders, apis),
	}, nil
}

func buildShareTree(folders []*ent.Folder, apis []*ent.API) []ShareTreeNode {
	childrenMap := make(map[int64][]*ent.Folder)
	for _, folder := range folders {
		childrenMap[folder.ParentID] = append(childrenMap[folder.ParentID], folder)
	}

	apisByFolder := make(map[int64][]*ent.API)
	folderByID := make(map[int64]*ent.Folder, len(folders))
	for _, folder := range folders {
		folderByID[folder.ID] = folder
	}
	for _, api := range apis {
		apisByFolder[api.FolderID] = append(apisByFolder[api.FolderID], api)
	}

	for parentID := range childrenMap {
		sort.Slice(childrenMap[parentID], func(i, j int) bool {
			left := childrenMap[parentID][i]
			right := childrenMap[parentID][j]
			if left.SortOrder != right.SortOrder {
				return left.SortOrder < right.SortOrder
			}
			return left.ID < right.ID
		})
	}
	for folderID := range apisByFolder {
		sort.Slice(apisByFolder[folderID], func(i, j int) bool {
			left := apisByFolder[folderID][i]
			right := apisByFolder[folderID][j]
			if left.SortOrder != right.SortOrder {
				return left.SortOrder < right.SortOrder
			}
			return left.ID < right.ID
		})
	}

	var build func(parentID int64) []ShareTreeNode
	build = func(parentID int64) []ShareTreeNode {
		nodes := make([]ShareTreeNode, 0, len(childrenMap[parentID])+len(apisByFolder[parentID]))
		for _, folder := range childrenMap[parentID] {
			children := build(folder.ID)
			for _, api := range apisByFolder[folder.ID] {
				children = append(children, ShareTreeNode{
					ID:     api.ID,
					Name:   api.Name,
					Type:   "api",
					Method: api.Method,
					URL:    api.URL,
					Status: api.Status,
				})
			}
			nodes = append(nodes, ShareTreeNode{
				ID:       folder.ID,
				Name:     folder.Name,
				Type:     "folder",
				Children: children,
			})
		}
		if parentID == 0 {
			for _, api := range apisByFolder[0] {
				nodes = append(nodes, ShareTreeNode{
					ID:     api.ID,
					Name:   api.Name,
					Type:   "api",
					Method: api.Method,
					URL:    api.URL,
					Status: api.Status,
				})
			}
			for folderID, folderApis := range apisByFolder {
				if folderID == 0 || folderByID[folderID] != nil {
					continue
				}
				for _, api := range folderApis {
					nodes = append(nodes, ShareTreeNode{
						ID:     api.ID,
						Name:   api.Name,
						Type:   "api",
						Method: api.Method,
						URL:    api.URL,
						Status: api.Status,
					})
				}
			}
		}
		return nodes
	}

	return build(0)
}

func (Service) AccessDetail(ctx context.Context, params AccessDetailRequest) (*interfaces.DetailItem, error) {
	row, err := Repo{}.LoadEnabledShare(ctx, params.ShareCode, params.Password)
	if err != nil {
		return nil, err
	}

	allowed := false
	for _, item := range row.Edges.Items {
		if item.InterfaceID == params.InterfaceID {
			allowed = true
			break
		}
	}
	if !allowed {
		return nil, bizerr.New("该接口未包含在此分享中")
	}

	return interfaces.LoadDetail(ctx, row.ProjectID, params.InterfaceID)
}
