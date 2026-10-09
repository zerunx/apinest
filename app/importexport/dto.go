package importexport

import (
	"encoding/json"

	"nest-api/app/interfaces"
)

const (
	FormatApinest = "apinest"
	Version       = 1
	SourceApinest = "apinest"
	SourcePostman = "postman"
	SourceOpenAPI = "openapi"
	SourceApiPost = "apipost"
	SourceEolink  = "eolink"
	SourceApifox  = "apifox"
	SourceYApi    = "yapi"
	ScopeProject  = "project"
	ScopeFolder   = "folder"
)

type ExportRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	FolderID    int64 `form:"folder_id" json:"folder_id"`
}

type ImportRequest struct {
	WorkspaceID int64           `json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64           `json:"project_id" binding:"required,min=1"`
	Source      string          `json:"source" binding:"required"`
	Document    json.RawMessage `json:"document" binding:"required"`
}

type ImportResult struct {
	Folders int `json:"folders"`
	APIs    int `json:"apis"`
}

type Document struct {
	Format     string      `json:"format"`
	Version    int         `json:"version"`
	ExportedAt string      `json:"exported_at"`
	Scope      string      `json:"scope"`
	Project    ProjectMeta `json:"project"`
	Tree       []Node      `json:"tree"`
}

type ProjectMeta struct {
	Name string `json:"name"`
}

type Node struct {
	Type             string                           `json:"type"`
	Name             string                           `json:"name"`
	Children         []Node                           `json:"children,omitempty"`
	Method           string                           `json:"method,omitempty"`
	Protocol         string                           `json:"protocol,omitempty"`
	URL              string                           `json:"url,omitempty"`
	Status           uint8                            `json:"status,omitempty"`
	RequestHeaders   []interfaces.ParamItem           `json:"request_headers,omitempty"`
	RequestBody      interfaces.RequestBodyConfig     `json:"request_body,omitempty"`
	QueryParams      []interfaces.ParamItem           `json:"query_params,omitempty"`
	ResponseHeaders  []interfaces.ResponseHeaderItem  `json:"response_headers,omitempty"`
	ResponseResults  []interfaces.ResponseResultItem  `json:"response_results,omitempty"`
	ResponseExamples []interfaces.ResponseExampleItem `json:"response_examples,omitempty"`
	GRPC             json.RawMessage                  `json:"grpc,omitempty"`
}
