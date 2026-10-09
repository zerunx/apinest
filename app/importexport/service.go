package importexport

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"time"
	"unicode/utf8"

	"nest-api/app/folder"
	"nest-api/app/interfaces"
	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	bizerr "nest-api/pkg/errors"
)

type Service struct{}

func (Service) Export(ctx context.Context, userID int64, params ExportRequest) (*Document, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}
	proj, err := project.Repo{}.Get(ctx, params.WorkspaceID, params.ProjectID)
	if err != nil {
		return nil, err
	}

	folders, err := folder.Repo{}.ListByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}
	apis, err := folder.Repo{}.ListAPIsByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}

	scope := ScopeProject
	treeFolders := folders
	treeAPIs := apis
	folderName := ""

	if params.FolderID > 0 {
		root, err := folder.Repo{}.Get(ctx, params.ProjectID, params.FolderID)
		if err != nil {
			return nil, err
		}
		scope = ScopeFolder
		folderName = root.Name
		allowed := descendantSet(params.FolderID, folders)
		treeFolders = filterFolders(folders, allowed)
		treeAPIs = filterAPIs(apis, allowed)
	}

	details := make(map[int64]*interfaces.DetailItem, len(treeAPIs))
	for _, item := range treeAPIs {
		detail, err := interfaces.LoadDetail(ctx, params.ProjectID, item.ID)
		if err != nil {
			return nil, err
		}
		details[item.ID] = detail
	}

	doc := &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      scope,
		Project:    ProjectMeta{Name: proj.Name},
	}
	if scope == ScopeFolder {
		doc.Tree = []Node{{
			Type:     "folder",
			Name:     folderName,
			Children: buildExportTree(treeFolders, treeAPIs, details, params.FolderID),
		}}
	} else {
		doc.Tree = buildExportTree(treeFolders, treeAPIs, details, 0)
	}
	return doc, nil
}

func (Service) Import(ctx context.Context, userID int64, params ImportRequest) (*ImportResult, error) {
	source := strings.ToLower(strings.TrimSpace(params.Source))
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectCreate); err != nil {
		return nil, err
	}

	doc, err := parseImportDocument(source, params.Document)
	if err != nil {
		return nil, err
	}

	folders, err := folder.Repo{}.ListByProject(ctx, params.ProjectID)
	if err != nil {
		return nil, err
	}
	usedNames := map[int64]map[string]struct{}{}
	for _, row := range folders {
		set := usedNames[row.ParentID]
		if set == nil {
			set = map[string]struct{}{}
			usedNames[row.ParentID] = set
		}
		set[row.Name] = struct{}{}
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return nil, err
	}

	result := &ImportResult{}
	folderSort, err := folder.Repo{}.NextSortOrderTx(ctx, tx, params.ProjectID, 0)
	if err != nil {
		_ = tx.Rollback()
		return nil, err
	}

	if err := importNodes(ctx, tx, userID, params.ProjectID, 0, doc.Tree, usedNames, &folderSort, result); err != nil {
		_ = tx.Rollback()
		return nil, err
	}
	if err := tx.Commit(); err != nil {
		return nil, err
	}
	return result, nil
}

func parseImportDocument(source string, raw json.RawMessage) (Document, error) {
	switch source {
	case SourceApinest:
		var doc Document
		if err := json.Unmarshal(raw, &doc); err != nil {
			return Document{}, bizerr.New("import file is not valid JSON")
		}
		if err := validateDocument(doc); err != nil {
			return Document{}, err
		}
		return doc, nil
	case SourcePostman:
		doc, err := ConvertPostmanCollection(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	case SourceOpenAPI:
		doc, err := ConvertOpenAPIDocument(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	case SourceApiPost:
		doc, err := ConvertApiPostDocument(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	case SourceEolink:
		doc, err := ConvertEolinkDocument(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	case SourceApifox:
		doc, err := ConvertApifoxDocument(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	case SourceYApi:
		doc, err := ConvertYApiDocument(raw)
		if err != nil {
			return Document{}, err
		}
		return *doc, nil
	default:
		return Document{}, bizerr.New("this import format is not supported yet")
	}
}

func validateDocument(doc Document) error {
	if !strings.EqualFold(strings.TrimSpace(doc.Format), FormatApinest) {
		return bizerr.New("not an ApiNest export file")
	}
	if doc.Version != Version {
		return bizerr.New("unsupported export version")
	}
	return nil
}

func importNodes(
	ctx context.Context,
	tx *ent.Tx,
	userID, projectID, parentID int64,
	nodes []Node,
	usedNames map[int64]map[string]struct{},
	folderSort *int,
	result *ImportResult,
) error {
	apiSort, err := interfaces.Repo{}.NextSortOrderTx(ctx, tx, projectID, parentID)
	if err != nil {
		return err
	}

	var rootAPIs []Node
	for _, node := range nodes {
		switch strings.ToLower(strings.TrimSpace(node.Type)) {
		case "folder":
			name := uniqueName(usedNames, parentID, node.Name, "未命名文件夹")
			row, err := folder.Repo{}.CreateTx(ctx, tx, projectID, parentID, userID, name, *folderSort)
			if err != nil {
				return err
			}
			*folderSort++
			result.Folders++
			childFolderSort := 0
			if err := importNodes(ctx, tx, userID, projectID, row.ID, node.Children, usedNames, &childFolderSort, result); err != nil {
				return err
			}
		case "api":
			if parentID == 0 {
				rootAPIs = append(rootAPIs, node)
				continue
			}
			if err := insertAPI(ctx, tx, userID, projectID, parentID, apiSort, node); err != nil {
				return err
			}
			apiSort++
			result.APIs++
		}
	}

	if len(rootAPIs) == 0 {
		return nil
	}

	ungrouped := uniqueName(usedNames, 0, "未分组", "未分组")
	row, err := folder.Repo{}.CreateTx(ctx, tx, projectID, 0, userID, ungrouped, *folderSort)
	if err != nil {
		return err
	}
	*folderSort++
	result.Folders++
	childSort := 0
	for _, node := range rootAPIs {
		if err := insertAPI(ctx, tx, userID, projectID, row.ID, childSort, node); err != nil {
			return err
		}
		childSort++
		result.APIs++
	}
	return nil
}

func insertAPI(ctx context.Context, tx *ent.Tx, userID, projectID, folderID int64, sortOrder int, node Node) error {
	params := interfaces.CreateRequest{
		Name:             node.Name,
		Method:           node.Method,
		Protocol:         node.Protocol,
		URL:              node.URL,
		Status:           node.Status,
		RequestHeaders:   node.RequestHeaders,
		RequestBody:      node.RequestBody,
		QueryParams:      node.QueryParams,
		ResponseHeaders:  node.ResponseHeaders,
		ResponseResults:  node.ResponseResults,
		ResponseExamples: node.ResponseExamples,
	}
	_, err := interfaces.InsertInTx(ctx, tx, userID, projectID, folderID, sortOrder, params)
	return err
}

func uniqueName(used map[int64]map[string]struct{}, parentID int64, name, fallback string) string {
	set := used[parentID]
	if set == nil {
		set = map[string]struct{}{}
		used[parentID] = set
	}
	base := clipName(strings.TrimSpace(name), 100)
	if base == "" {
		base = fallback
	}
	candidate := base
	for i := 2; ; i++ {
		if _, ok := set[candidate]; !ok {
			set[candidate] = struct{}{}
			return candidate
		}
		suffix := fmt.Sprintf(" (%d)", i)
		limit := 100 - utf8.RuneCountInString(suffix)
		if limit < 1 {
			limit = 1
		}
		candidate = clipName(base, limit) + suffix
	}
}

func clipName(name string, maxRunes int) string {
	if maxRunes <= 0 {
		return ""
	}
	if utf8.RuneCountInString(name) <= maxRunes {
		return name
	}
	return string([]rune(name)[:maxRunes])
}

func descendantSet(rootID int64, folders []*ent.Folder) map[int64]struct{} {
	children := map[int64][]int64{}
	for _, row := range folders {
		children[row.ParentID] = append(children[row.ParentID], row.ID)
	}
	set := map[int64]struct{}{rootID: {}}
	queue := []int64{rootID}
	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]
		for _, id := range children[current] {
			if _, ok := set[id]; ok {
				continue
			}
			set[id] = struct{}{}
			queue = append(queue, id)
		}
	}
	return set
}

func filterFolders(folders []*ent.Folder, allowed map[int64]struct{}) []*ent.Folder {
	out := make([]*ent.Folder, 0, len(folders))
	for _, row := range folders {
		if _, ok := allowed[row.ID]; ok {
			out = append(out, row)
		}
	}
	return out
}

func filterAPIs(apis []*ent.API, allowed map[int64]struct{}) []*ent.API {
	out := make([]*ent.API, 0, len(apis))
	for _, row := range apis {
		if _, ok := allowed[row.FolderID]; ok {
			out = append(out, row)
		}
	}
	return out
}

func buildExportTree(folders []*ent.Folder, apis []*ent.API, details map[int64]*interfaces.DetailItem, parentID int64) []Node {
	folderChildren := map[int64][]*ent.Folder{}
	for _, row := range folders {
		folderChildren[row.ParentID] = append(folderChildren[row.ParentID], row)
	}
	apiChildren := map[int64][]*ent.API{}
	for _, row := range apis {
		apiChildren[row.FolderID] = append(apiChildren[row.FolderID], row)
	}

	nodes := make([]Node, 0, len(folderChildren[parentID])+len(apiChildren[parentID]))
	for _, row := range folderChildren[parentID] {
		nodes = append(nodes, Node{
			Type:     "folder",
			Name:     row.Name,
			Children: buildExportTree(folders, apis, details, row.ID),
		})
	}
	for _, row := range apiChildren[parentID] {
		nodes = append(nodes, apiNode(row, details[row.ID]))
	}
	return nodes
}

func apiNode(row *ent.API, detail *interfaces.DetailItem) Node {
	node := Node{
		Type:     "api",
		Name:     row.Name,
		Method:   row.Method,
		Protocol: row.Protocol,
		URL:      row.URL,
		Status:   row.Status,
	}
	if detail != nil {
		node.Name = detail.Name
		node.Method = detail.Method
		node.Protocol = detail.Protocol
		node.URL = detail.URL
		node.Status = detail.Status
		node.RequestHeaders = detail.RequestHeaders
		node.RequestBody = detail.RequestBody
		node.QueryParams = detail.QueryParams
		node.ResponseHeaders = detail.ResponseHeaders
		node.ResponseResults = detail.ResponseResults
		node.ResponseExamples = detail.ResponseExamples
	}
	return node
}
