package folder

import (
	"context"
	"fmt"
	"sort"

	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/ent"
)

type Service struct{}

func (Service) Tree(ctx context.Context, userID int64, params ProjectScopeRequest) ([]TreeNode, error) {
	if err := requireProject(ctx, userID, params); err != nil {
		return nil, err
	}

	folders, err := Repo{}.ListByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}

	ifaces, err := Repo{}.ListAPIsByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}

	return buildTree(folders, ifaces), nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) (int64, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectCreate); err != nil {
		return 0, err
	}
	if params.ParentID > 0 {
		if err := EnsureExists(ctx, params.ProjectID, params.ParentID); err != nil {
			return 0, err
		}
	}

	row, err := Repo{}.Create(ctx, params.ProjectID, params.ParentID, userID, params.Name)
	if err != nil {
		return 0, err
	}
	return row.ID, nil
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.FolderID); err != nil {
		return err
	}

	return (Repo{}).UpdateName(ctx, params.FolderID, params.Name)
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectDelete); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.FolderID); err != nil {
		return err
	}

	folders, err := Repo{}.ListByProject(ctx, params.ProjectID)
	if err != nil {
		return err
	}

	ifaces, err := Repo{}.ListAPIsByProject(ctx, params.ProjectID)
	if err != nil {
		return err
	}

	folderIDs := collectDescendants(params.FolderID, folders)
	folderIDSet := make(map[int64]struct{}, len(folderIDs))
	for _, id := range folderIDs {
		folderIDSet[id] = struct{}{}
	}

	repo := Repo{}
	for _, item := range ifaces {
		if _, ok := folderIDSet[item.FolderID]; ok {
			if err := repo.DeleteAPI(ctx, item.ID); err != nil {
				return err
			}
		}
	}

	for i := len(folderIDs) - 1; i >= 0; i-- {
		if err := repo.DeleteOne(ctx, folderIDs[i]); err != nil {
			return err
		}
	}
	return nil
}

func requireProject(ctx context.Context, userID int64, params ProjectScopeRequest) error {
	_, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectRead)
	return err
}

func buildTree(folders []*ent.Folder, ifaces []*ent.API) []TreeNode {
	folderChildren := make(map[int64][]*ent.Folder)
	for _, folder := range folders {
		folderChildren[folder.ParentID] = append(folderChildren[folder.ParentID], folder)
	}

	ifaceChildren := make(map[int64][]*ent.API)
	for _, item := range ifaces {
		ifaceChildren[item.FolderID] = append(ifaceChildren[item.FolderID], item)
	}

	for parentID := range folderChildren {
		sort.Slice(folderChildren[parentID], func(i, j int) bool {
			left := folderChildren[parentID][i]
			right := folderChildren[parentID][j]
			if left.SortOrder != right.SortOrder {
				return left.SortOrder < right.SortOrder
			}
			return left.ID < right.ID
		})
	}
	for folderID := range ifaceChildren {
		sort.Slice(ifaceChildren[folderID], func(i, j int) bool {
			left := ifaceChildren[folderID][i]
			right := ifaceChildren[folderID][j]
			if left.SortOrder != right.SortOrder {
				return left.SortOrder < right.SortOrder
			}
			return left.ID < right.ID
		})
	}

	var buildFolders func(parentID int64) []TreeNode
	buildFolders = func(parentID int64) []TreeNode {
		children := folderChildren[parentID]
		nodes := make([]TreeNode, 0, len(children))
		for _, folder := range children {
			node := TreeNode{
				ID:        folderIDKey(folder.ID),
				ProjectID: folder.ProjectID,
				Name:      folder.Name,
				Type:      "folder",
			}
			subFolders := buildFolders(folder.ID)
			apiNodes := make([]TreeNode, 0, len(ifaceChildren[folder.ID]))
			for _, item := range ifaceChildren[folder.ID] {
				apiNodes = append(apiNodes, TreeNode{
					ID:        interfaceIDKey(item.ID),
					ProjectID: item.ProjectID,
					Name:      item.Name,
					Type:      "api",
					Method:    item.Method,
				})
			}
			node.Children = append(subFolders, apiNodes...)
			nodes = append(nodes, node)
		}
		return nodes
	}

	return buildFolders(0)
}

func collectDescendants(rootID int64, folders []*ent.Folder) []int64 {
	childrenMap := make(map[int64][]int64)
	for _, folder := range folders {
		childrenMap[folder.ParentID] = append(childrenMap[folder.ParentID], folder.ID)
	}

	result := []int64{rootID}
	queue := []int64{rootID}
	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]
		for _, childID := range childrenMap[current] {
			result = append(result, childID)
			queue = append(queue, childID)
		}
	}
	return result
}

func folderIDKey(id int64) string {
	return fmt.Sprintf("folder-%d", id)
}

func interfaceIDKey(id int64) string {
	return fmt.Sprintf("api-%d", id)
}
