package interfaces

import (
	"context"
	"strings"

	"nest-api/internal/database"
	"nest-api/internal/ent"
	entapi "nest-api/internal/ent/api"
	entbodyfield "nest-api/internal/ent/interfacebodyfield"
	entexample "nest-api/internal/ent/interfaceexample"
	entfield "nest-api/internal/ent/interfacefield"
	entheader "nest-api/internal/ent/interfaceheader"
	entquery "nest-api/internal/ent/interfacequeryparam"
	entreqheader "nest-api/internal/ent/interfacerequestheader"
	entresult "nest-api/internal/ent/interfaceresult"
	bizerr "nest-api/pkg/errors"
)

type Repo struct{}

// EnsureExists 校验接口是否存在于指定项目。
func EnsureExists(ctx context.Context, projectID, interfaceID int64) error {
	return (Repo{}).EnsureExists(ctx, projectID, interfaceID)
}

func (Repo) EnsureExists(ctx context.Context, projectID, interfaceID int64) error {
	exists, err := database.DB.API.
		Query().
		Where(
			entapi.IDEQ(interfaceID),
			entapi.ProjectIDEQ(projectID),
		).
		Exist(ctx)
	if err != nil {
		return err
	}
	if !exists {
		return bizerr.New("接口不存在")
	}
	return nil
}

// CountInProject 统计给定接口 ID 中属于指定项目的数量。
func (Repo) CountInProject(ctx context.Context, projectID int64, ids []int64) (int, error) {
	if len(ids) == 0 {
		return 0, nil
	}
	return database.DB.API.
		Query().
		Where(
			entapi.ProjectIDEQ(projectID),
			entapi.IDIn(ids...),
		).
		Count(ctx)
}

func (Repo) NextSortOrder(ctx context.Context, projectID, folderID int64) (int, error) {
	return nextAPISortOrder(ctx, database.DB.API.Query(), projectID, folderID)
}

func (Repo) NextSortOrderTx(ctx context.Context, tx *ent.Tx, projectID, folderID int64) (int, error) {
	return nextAPISortOrder(ctx, tx.API.Query(), projectID, folderID)
}

func nextAPISortOrder(ctx context.Context, q *ent.APIQuery, projectID, folderID int64) (int, error) {
	rows, err := q.
		Where(
			entapi.ProjectIDEQ(projectID),
			entapi.FolderIDEQ(folderID),
		).
		Order(ent.Desc(entapi.FieldSortOrder)).
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

func (Repo) ListByProject(ctx context.Context, projectID int64) ([]*ent.API, error) {
	return database.DB.API.
		Query().
		Where(entapi.ProjectIDEQ(projectID)).
		WithFolder().
		WithUpdater().
		Order(ent.Asc(entapi.FieldSortOrder), ent.Asc(entapi.FieldID)).
		All(ctx)
}

func (Repo) ListByFolder(ctx context.Context, projectID, folderID int64) ([]*ent.API, error) {
	return database.DB.API.
		Query().
		Where(
			entapi.ProjectIDEQ(projectID),
			entapi.FolderIDEQ(folderID),
		).
		All(ctx)
}

func (Repo) GetWithDetails(ctx context.Context, projectID, interfaceID int64) (*ent.API, error) {
	return database.DB.API.
		Query().
		Where(
			entapi.IDEQ(interfaceID),
			entapi.ProjectIDEQ(projectID),
		).
		WithFolder().
		WithUpdater().
		WithRequestHeaders(func(q *ent.InterfaceRequestHeaderQuery) {
			q.Order(ent.Asc(entreqheader.FieldSortOrder), ent.Asc(entreqheader.FieldID))
		}).
		WithQueryParams(func(q *ent.InterfaceQueryParamQuery) {
			q.Order(ent.Asc(entquery.FieldSortOrder), ent.Asc(entquery.FieldID))
		}).
		WithBodyFields(func(q *ent.InterfaceBodyFieldQuery) {
			q.Order(ent.Asc(entbodyfield.FieldSortOrder), ent.Asc(entbodyfield.FieldID))
		}).
		WithResponseHeaders(func(q *ent.InterfaceHeaderQuery) {
			q.Order(ent.Asc(entheader.FieldSortOrder), ent.Asc(entheader.FieldID))
		}).
		WithResponseResults(func(q *ent.InterfaceResultQuery) {
			q.Order(ent.Asc(entresult.FieldSortOrder), ent.Asc(entresult.FieldID)).
				WithFields(func(fq *ent.InterfaceFieldQuery) {
					fq.Order(ent.Asc(entfield.FieldSortOrder), ent.Asc(entfield.FieldID))
				})
		}).
		WithResponseExamples(func(q *ent.InterfaceExampleQuery) {
			q.Order(ent.Asc(entexample.FieldSortOrder), ent.Asc(entexample.FieldID))
		}).
		Only(ctx)
}

func (Repo) Create(ctx context.Context, projectID, folderID, userID int64, name, method, url string, status uint8, sortOrder int) (*ent.API, error) {
	return database.DB.API.
		Create().
		SetProjectID(projectID).
		SetFolderID(folderID).
		SetName(name).
		SetMethod(method).
		SetURL(url).
		SetStatus(status).
		SetSortOrder(sortOrder).
		SetCreatedBy(userID).
		SetUpdatedBy(userID).
		Save(ctx)
}

func (Repo) Delete(ctx context.Context, interfaceID int64) error {
	return database.DB.API.DeleteOneID(interfaceID).Exec(ctx)
}

func (Repo) ReplaceRequestHeaders(ctx context.Context, tx *ent.Tx, interfaceID int64, headers []ParamItem) error {
	if _, err := tx.InterfaceRequestHeader.Delete().Where(entreqheader.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}
	for index, header := range headers {
		if strings.TrimSpace(header.Name) == "" {
			continue
		}
		headerType := header.Type
		if headerType == "" {
			headerType = "string"
		}
		if _, err := tx.InterfaceRequestHeader.
			Create().
			SetInterfaceID(interfaceID).
			SetName(header.Name).
			SetType(headerType).
			SetRequired(header.Required).
			SetDescription(header.Description).
			SetExample(header.Example).
			SetSortOrder(index).
			Save(ctx); err != nil {
			return err
		}
	}
	return nil
}

func (Repo) ReplaceQueryParams(ctx context.Context, tx *ent.Tx, interfaceID int64, params []ParamItem) error {
	if _, err := tx.InterfaceQueryParam.Delete().Where(entquery.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}
	for index, param := range params {
		if strings.TrimSpace(param.Name) == "" {
			continue
		}
		paramType := param.Type
		if paramType == "" {
			paramType = "string"
		}
		if _, err := tx.InterfaceQueryParam.
			Create().
			SetInterfaceID(interfaceID).
			SetName(param.Name).
			SetType(paramType).
			SetRequired(param.Required).
			SetDescription(param.Description).
			SetExample(param.Example).
			SetSortOrder(index).
			Save(ctx); err != nil {
			return err
		}
	}
	return nil
}

func (Repo) ReplaceBodyFields(ctx context.Context, tx *ent.Tx, interfaceID int64, fields []BodyFieldItem) error {
	if _, err := tx.InterfaceBodyField.Delete().Where(entbodyfield.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}
	return (Repo{}).saveBodyFields(ctx, tx, interfaceID, 0, fields)
}

func (Repo) saveBodyFields(ctx context.Context, tx *ent.Tx, interfaceID, parentID int64, fields []BodyFieldItem) error {
	for index, fieldItem := range fields {
		if strings.TrimSpace(fieldItem.Name) == "" {
			continue
		}
		fieldType := fieldItem.Type
		if fieldType == "" {
			fieldType = "string"
		}

		saved, err := tx.InterfaceBodyField.
			Create().
			SetInterfaceID(interfaceID).
			SetParentID(parentID).
			SetName(fieldItem.Name).
			SetType(fieldType).
			SetRequired(fieldItem.Required).
			SetDescription(fieldItem.Description).
			SetExample(fieldItem.Example).
			SetSortOrder(index).
			Save(ctx)
		if err != nil {
			return err
		}

		if len(fieldItem.Children) > 0 {
			if err := (Repo{}).saveBodyFields(ctx, tx, interfaceID, saved.ID, fieldItem.Children); err != nil {
				return err
			}
		}
	}
	return nil
}

func (Repo) ReplaceResponseHeaders(ctx context.Context, tx *ent.Tx, interfaceID int64, headers []ResponseHeaderItem) error {
	if _, err := tx.InterfaceHeader.Delete().Where(entheader.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}

	for index, header := range headers {
		if strings.TrimSpace(header.Name) == "" {
			continue
		}
		headerType := header.Type
		if headerType == "" {
			headerType = "string"
		}
		if _, err := tx.InterfaceHeader.
			Create().
			SetInterfaceID(interfaceID).
			SetName(header.Name).
			SetType(headerType).
			SetRequired(header.Required).
			SetDescription(header.Description).
			SetExample(header.Example).
			SetSortOrder(index).
			Save(ctx); err != nil {
			return err
		}
	}
	return nil
}

func (Repo) ReplaceResponseResults(ctx context.Context, tx *ent.Tx, interfaceID int64, results []ResponseResultItem) error {
	existingResults, err := tx.InterfaceResult.
		Query().
		Where(entresult.InterfaceIDEQ(interfaceID)).
		All(ctx)
	if err != nil {
		return err
	}
	for _, row := range existingResults {
		if _, err := tx.InterfaceField.Delete().Where(entfield.ResultIDEQ(row.ID)).Exec(ctx); err != nil {
			return err
		}
	}
	if _, err := tx.InterfaceResult.Delete().Where(entresult.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}

	for index, result := range results {
		format := result.Format
		if format == "" {
			format = "JSON"
		}
		dataType := result.DataType
		if dataType == "" {
			dataType = "Object"
		}
		statusCode := result.StatusCode
		if statusCode == 0 {
			statusCode = 200
		}

		saved, err := tx.InterfaceResult.
			Create().
			SetInterfaceID(interfaceID).
			SetName(result.Name).
			SetStatusCode(statusCode).
			SetFormat(format).
			SetDataType(dataType).
			SetSortOrder(index).
			Save(ctx)
		if err != nil {
			return err
		}

		if err := (Repo{}).saveResponseFields(ctx, tx, saved.ID, 0, result.Fields); err != nil {
			return err
		}
	}
	return nil
}

func (Repo) saveResponseFields(ctx context.Context, tx *ent.Tx, resultID, parentID int64, fields []ResponseFieldItem) error {
	for index, fieldItem := range fields {
		if strings.TrimSpace(fieldItem.Name) == "" {
			continue
		}
		fieldType := fieldItem.Type
		if fieldType == "" {
			fieldType = "string"
		}

		saved, err := tx.InterfaceField.
			Create().
			SetResultID(resultID).
			SetParentID(parentID).
			SetName(fieldItem.Name).
			SetType(fieldType).
			SetRequired(fieldItem.Required).
			SetDescription(fieldItem.Description).
			SetMock(fieldItem.Mock).
			SetExample(fieldItem.Example).
			SetSortOrder(index).
			Save(ctx)
		if err != nil {
			return err
		}

		if len(fieldItem.Children) > 0 {
			if err := (Repo{}).saveResponseFields(ctx, tx, resultID, saved.ID, fieldItem.Children); err != nil {
				return err
			}
		}
	}
	return nil
}

func (Repo) ReplaceResponseExamples(ctx context.Context, tx *ent.Tx, interfaceID int64, examples []ResponseExampleItem) error {
	if _, err := tx.InterfaceExample.Delete().Where(entexample.InterfaceIDEQ(interfaceID)).Exec(ctx); err != nil {
		return err
	}

	for index, example := range examples {
		statusCode := example.StatusCode
		if statusCode == 0 {
			statusCode = 200
		}
		contentType := example.ContentType
		if contentType == "" {
			contentType = "application/json"
		}
		name := example.Name
		if name == "" {
			name = "示例"
		}

		if _, err := tx.InterfaceExample.
			Create().
			SetInterfaceID(interfaceID).
			SetName(name).
			SetStatusCode(statusCode).
			SetContentType(contentType).
			SetRaw(example.Raw).
			SetSortOrder(index).
			Save(ctx); err != nil {
			return err
		}
	}
	return nil
}
