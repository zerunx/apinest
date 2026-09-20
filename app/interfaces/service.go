package interfaces

import (
	"context"
	"strings"
	"unicode/utf8"

	"nest-api/app/folder"
	"nest-api/app/project"
	"nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
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
		items = append(items, toItem(row))
	}
	return items, nil
}

func (Service) Detail(ctx context.Context, userID int64, params DetailRequest) (*DetailItem, error) {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectRead); err != nil {
		return nil, err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.InterfaceID); err != nil {
		return nil, err
	}

	return LoadDetail(ctx, params.ProjectID, params.InterfaceID)
}

func LoadDetail(ctx context.Context, projectID, interfaceID int64) (*DetailItem, error) {
	row, err := Repo{}.GetWithDetails(ctx, projectID, interfaceID)
	if err != nil {
		return nil, err
	}

	return &DetailItem{
		Item:           toItem(row),
		RequestHeaders: buildParamItems(row.Edges.RequestHeaders),
		RequestBody: RequestBodyConfig{
			Format:   row.RequestBodyFormat,
			DataType: row.RequestBodyDataType,
			Raw:      row.RequestBodyRaw,
			Fields:   buildBodyFieldTree(row.Edges.BodyFields),
		},
		QueryParams:      buildQueryParamItems(row.Edges.QueryParams),
		ResponseHeaders:  buildResponseHeaders(row.Edges.ResponseHeaders),
		ResponseResults:  buildResponseResults(row.Edges.ResponseResults),
		ResponseExamples: buildResponseExamples(row.Edges.ResponseExamples),
	}, nil
}

func (Service) Create(ctx context.Context, userID int64, params CreateRequest) (int64, error) {
	if userID <= 0 {
		return 0, bizerr.New("not logged in")
	}
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectCreate); err != nil {
		return 0, err
	}
	if err := folder.EnsureExists(ctx, params.ProjectID, params.FolderID); err != nil {
		return 0, err
	}

	sortOrder, err := Repo{}.NextSortOrder(ctx, params.ProjectID, params.FolderID)
	if err != nil {
		return 0, err
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return 0, err
	}

	id, err := InsertInTx(ctx, tx, userID, params.ProjectID, params.FolderID, sortOrder, params)
	if err != nil {
		_ = tx.Rollback()
		return 0, err
	}
	if err := tx.Commit(); err != nil {
		return 0, err
	}
	return id, nil
}

func InsertInTx(ctx context.Context, tx *ent.Tx, userID, projectID, folderID int64, sortOrder int, params CreateRequest) (int64, error) {
	method := strings.ToUpper(strings.TrimSpace(params.Method))
	if method == "" {
		method = "GET"
	}
	if len(method) > 10 {
		method = method[:10]
	}
	status := params.Status
	if status == 0 {
		status = 3
	}

	bodyFormat := params.RequestBody.Format
	if bodyFormat == "" {
		bodyFormat = "json"
	}
	bodyDataType := params.RequestBody.DataType
	if bodyDataType == "" {
		bodyDataType = "Object"
	}

	name := strings.TrimSpace(params.Name)
	if name == "" {
		name = "New Interface"
	}
	if utf8.RuneCountInString(name) > 100 {
		name = string([]rune(name)[:100])
	}
	url := params.URL
	if len(url) > 500 {
		url = url[:500]
	}

	row, err := tx.API.
		Create().
		SetProjectID(projectID).
		SetFolderID(folderID).
		SetName(name).
		SetMethod(method).
		SetProtocol(normalizeProtocol(params.Protocol, method, url)).
		SetURL(url).
		SetStatus(status).
		SetSortOrder(sortOrder).
		SetRequestBodyFormat(bodyFormat).
		SetRequestBodyDataType(bodyDataType).
		SetRequestBodyRaw(params.RequestBody.Raw).
		SetCreatedBy(userID).
		SetUpdatedBy(userID).
		SetCreatorID(userID).
		SetUpdaterID(userID).
		Save(ctx)
	if err != nil {
		return 0, err
	}

	repo := Repo{}
	if err := repo.ReplaceRequestHeaders(ctx, tx, row.ID, params.RequestHeaders); err != nil {
		return 0, err
	}
	if err := repo.ReplaceQueryParams(ctx, tx, row.ID, params.QueryParams); err != nil {
		return 0, err
	}
	if err := repo.ReplaceBodyFields(ctx, tx, row.ID, params.RequestBody.Fields); err != nil {
		return 0, err
	}
	if err := repo.ReplaceResponseHeaders(ctx, tx, row.ID, params.ResponseHeaders); err != nil {
		return 0, err
	}
	if err := repo.ReplaceResponseResults(ctx, tx, row.ID, params.ResponseResults); err != nil {
		return 0, err
	}
	if err := repo.ReplaceResponseExamples(ctx, tx, row.ID, params.ResponseExamples); err != nil {
		return 0, err
	}
	return row.ID, nil
}

func (Service) Update(ctx context.Context, userID int64, params UpdateRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.InterfaceID); err != nil {
		return err
	}

	status := params.Status
	if status == 0 {
		status = 3
	}

	bodyFormat := params.RequestBody.Format
	if bodyFormat == "" {
		bodyFormat = "json"
	}
	bodyDataType := params.RequestBody.DataType
	if bodyDataType == "" {
		bodyDataType = "Object"
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return err
	}

	updater := tx.API.
		UpdateOneID(params.InterfaceID).
		SetName(params.Name).
		SetMethod(strings.ToUpper(params.Method)).
		SetProtocol(normalizeProtocol(params.Protocol, params.Method, params.URL)).
		SetURL(params.URL).
		SetStatus(status).
		SetRequestBodyFormat(bodyFormat).
		SetRequestBodyDataType(bodyDataType).
		SetRequestBodyRaw(params.RequestBody.Raw).
		SetUpdatedBy(userID)

	if params.FolderID > 0 {
		if err := folder.EnsureExists(ctx, params.ProjectID, params.FolderID); err != nil {
			_ = tx.Rollback()
			return err
		}
		updater = updater.SetFolderID(params.FolderID)
	}

	if err := updater.Exec(ctx); err != nil {
		_ = tx.Rollback()
		return err
	}

	repo := Repo{}
	if err := repo.ReplaceRequestHeaders(ctx, tx, params.InterfaceID, params.RequestHeaders); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err := repo.ReplaceQueryParams(ctx, tx, params.InterfaceID, params.QueryParams); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err := repo.ReplaceBodyFields(ctx, tx, params.InterfaceID, params.RequestBody.Fields); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err := repo.ReplaceResponseHeaders(ctx, tx, params.InterfaceID, params.ResponseHeaders); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err := repo.ReplaceResponseResults(ctx, tx, params.InterfaceID, params.ResponseResults); err != nil {
		_ = tx.Rollback()
		return err
	}
	if err := repo.ReplaceResponseExamples(ctx, tx, params.InterfaceID, params.ResponseExamples); err != nil {
		_ = tx.Rollback()
		return err
	}

	return tx.Commit()
}

func (Service) Delete(ctx context.Context, userID int64, params DeleteRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectDelete); err != nil {
		return err
	}
	if err := EnsureExists(ctx, params.ProjectID, params.InterfaceID); err != nil {
		return err
	}

	return (Repo{}).Delete(ctx, params.InterfaceID)
}

func (Service) Reorder(ctx context.Context, userID int64, params ReorderRequest) error {
	if _, err := project.RequireAccess(ctx, userID, params.WorkspaceID, params.ProjectID, workspace.ActionProjectUpdate); err != nil {
		return err
	}
	if err := folder.EnsureExists(ctx, params.ProjectID, params.FolderID); err != nil {
		return err
	}

	rows, err := Repo{}.ListByFolder(ctx, params.ProjectID, params.FolderID)
	if err != nil {
		return err
	}

	if len(params.InterfaceIDs) != len(rows) {
		return bizerr.New("interface list is incomplete")
	}

	existingIDs := make(map[int64]struct{}, len(rows))
	for _, row := range rows {
		existingIDs[row.ID] = struct{}{}
	}

	seen := make(map[int64]struct{}, len(params.InterfaceIDs))
	for _, id := range params.InterfaceIDs {
		if _, ok := existingIDs[id]; !ok {
			return bizerr.New("interface does not exist or does not belong to this folder")
		}
		if _, dup := seen[id]; dup {
			return bizerr.New("duplicate interfaces in list")
		}
		seen[id] = struct{}{}
	}

	tx, err := database.DB.Tx(ctx)
	if err != nil {
		return err
	}

	for index, id := range params.InterfaceIDs {
		if err := tx.API.UpdateOneID(id).SetSortOrder(index).Exec(ctx); err != nil {
			_ = tx.Rollback()
			return err
		}
	}

	return tx.Commit()
}

func normalizeProtocol(protocol, method, url string) string {
	switch strings.ToUpper(strings.TrimSpace(protocol)) {
	case "HTTP", "HTTPS", "WS", "TCP", "UDP", "GRPC":
		return strings.ToUpper(strings.TrimSpace(protocol))
	case "WSS":
		return "WS"
	case "RPC":
		return "GRPC"
	}
	switch strings.ToUpper(strings.TrimSpace(method)) {
	case "WS", "WSS":
		return "WS"
	case "TCP":
		return "TCP"
	case "UDP":
		return "UDP"
	case "GRPC", "RPC":
		return "GRPC"
	}
	if len(url) >= 8 && strings.EqualFold(url[:8], "https://") {
		return "HTTPS"
	}
	return "HTTP"
}

func toItem(row *ent.API) Item {
	folderName := ""
	if row.Edges.Folder != nil {
		folderName = row.Edges.Folder.Name
	}
	return Item{
		ID:            row.ID,
		ProjectID:     row.ProjectID,
		FolderID:      row.FolderID,
		Name:          row.Name,
		Method:        row.Method,
		Protocol:      normalizeProtocol(row.Protocol, row.Method, row.URL),
		URL:           row.URL,
		Status:        row.Status,
		FolderName:    folderName,
		CreatedBy:     row.CreatedBy,
		CreatedByName: workspace.UserDisplayName(row.Edges.Creator),
		UpdatedBy:     row.UpdatedBy,
		UpdatedByName: workspace.UserDisplayName(row.Edges.Updater),
		CreatedAt:     row.CreatedAt.Format(utils.DateTimeFormat),
		UpdatedAt:     row.UpdatedAt.Format(utils.DateTimeFormat),
	}
}

func buildParamItems(rows []*ent.InterfaceRequestHeader) []ParamItem {
	items := make([]ParamItem, 0, len(rows))
	for _, row := range rows {
		items = append(items, ParamItem{
			ID:          row.ID,
			Name:        row.Name,
			Type:        row.Type,
			Required:    row.Required,
			Enabled:     enabledPtr(row.Enabled),
			Description: row.Description,
			Example:     row.Example,
		})
	}
	return items
}

func buildQueryParamItems(rows []*ent.InterfaceQueryParam) []ParamItem {
	items := make([]ParamItem, 0, len(rows))
	for _, row := range rows {
		items = append(items, ParamItem{
			ID:          row.ID,
			Name:        row.Name,
			Type:        row.Type,
			Required:    row.Required,
			Enabled:     enabledPtr(row.Enabled),
			Description: row.Description,
			Example:     row.Example,
		})
	}
	return items
}

func buildBodyFieldTree(rows []*ent.InterfaceBodyField) []BodyFieldItem {
	if len(rows) == 0 {
		return []BodyFieldItem{}
	}

	children := make(map[int64][]*ent.InterfaceBodyField)
	for _, row := range rows {
		children[row.ParentID] = append(children[row.ParentID], row)
	}

	var build func(parentID int64) []BodyFieldItem
	build = func(parentID int64) []BodyFieldItem {
		list := children[parentID]
		items := make([]BodyFieldItem, 0, len(list))
		for _, row := range list {
			items = append(items, BodyFieldItem{
				ID:          row.ID,
				ParentID:    row.ParentID,
				Name:        row.Name,
				Type:        row.Type,
				Required:    row.Required,
				Enabled:     enabledPtr(row.Enabled),
				Description: row.Description,
				Example:     row.Example,
				Children:    build(row.ID),
			})
		}
		return items
	}
	return build(0)
}

func buildResponseHeaders(rows []*ent.InterfaceHeader) []ResponseHeaderItem {
	items := make([]ResponseHeaderItem, 0, len(rows))
	for _, row := range rows {
		items = append(items, ResponseHeaderItem{
			ID:          row.ID,
			Name:        row.Name,
			Type:        row.Type,
			Required:    row.Required,
			Description: row.Description,
			Example:     row.Example,
		})
	}
	return items
}

func buildResponseResults(rows []*ent.InterfaceResult) []ResponseResultItem {
	items := make([]ResponseResultItem, 0, len(rows))
	for _, row := range rows {
		items = append(items, ResponseResultItem{
			ID:         row.ID,
			Name:       row.Name,
			StatusCode: row.StatusCode,
			Format:     row.Format,
			DataType:   row.DataType,
			Fields:     buildResponseFieldTree(row.Edges.Fields),
		})
	}
	return items
}

func buildResponseFieldTree(rows []*ent.InterfaceField) []ResponseFieldItem {
	if len(rows) == 0 {
		return []ResponseFieldItem{}
	}

	children := make(map[int64][]*ent.InterfaceField)
	for _, row := range rows {
		children[row.ParentID] = append(children[row.ParentID], row)
	}

	var build func(parentID int64) []ResponseFieldItem
	build = func(parentID int64) []ResponseFieldItem {
		list := children[parentID]
		items := make([]ResponseFieldItem, 0, len(list))
		for _, row := range list {
			items = append(items, ResponseFieldItem{
				ID:          row.ID,
				ParentID:    row.ParentID,
				Name:        row.Name,
				Type:        row.Type,
				Required:    row.Required,
				Description: row.Description,
				Mock:        row.Mock,
				Example:     row.Example,
				Children:    build(row.ID),
			})
		}
		return items
	}
	return build(0)
}

func buildResponseExamples(rows []*ent.InterfaceExample) []ResponseExampleItem {
	items := make([]ResponseExampleItem, 0, len(rows))
	for _, row := range rows {
		items = append(items, ResponseExampleItem{
			ID:          row.ID,
			Name:        row.Name,
			StatusCode:  row.StatusCode,
			ContentType: row.ContentType,
			Raw:         row.Raw,
		})
	}
	return items
}
