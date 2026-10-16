package interfaces

type Item struct {
	ID            int64  `json:"id"`
	ProjectID     int64  `json:"project_id"`
	FolderID      int64  `json:"folder_id"`
	Name          string `json:"name"`
	Method        string `json:"method"`
	Protocol      string `json:"protocol,omitempty"`
	URL           string `json:"url"`
	Status        uint8  `json:"status"`
	FolderName    string `json:"folder_name,omitempty"`
	CreatedBy     int64  `json:"created_by"`
	CreatedByName string `json:"created_by_name,omitempty"`
	UpdatedBy     int64  `json:"updated_by"`
	UpdatedByName string `json:"updated_by_name,omitempty"`
	CreatedAt     string `json:"created_at"`
	UpdatedAt     string `json:"updated_at"`
}

type ParamItem struct {
	ID          int64  `json:"id,omitempty"`
	Name        string `json:"name"`
	Type        string `json:"type"`
	Required    bool   `json:"required"`
	Enabled     *bool  `json:"enabled,omitempty"`
	Description string `json:"description"`
	Example     string `json:"example"`
}

type BodyFieldItem struct {
	ID          int64           `json:"id,omitempty"`
	ParentID    int64           `json:"parent_id"`
	Name        string          `json:"name"`
	Type        string          `json:"type"`
	Required    bool            `json:"required"`
	Enabled     *bool           `json:"enabled,omitempty"`
	Description string          `json:"description"`
	Example     string          `json:"example"`
	Children    []BodyFieldItem `json:"children,omitempty"`
}

type RequestBodyConfig struct {
	Format   string          `json:"format"`
	DataType string          `json:"data_type"`
	Raw      string          `json:"raw"`
	Fields   []BodyFieldItem `json:"fields"`
}

type ResponseHeaderItem struct {
	ID          int64  `json:"id,omitempty"`
	Name        string `json:"name"`
	Type        string `json:"type"`
	Required    bool   `json:"required"`
	Description string `json:"description"`
	Example     string `json:"example"`
}

type ResponseFieldItem struct {
	ID          int64               `json:"id,omitempty"`
	ParentID    int64               `json:"parent_id"`
	Name        string              `json:"name"`
	Type        string              `json:"type"`
	Required    bool                `json:"required"`
	Description string              `json:"description"`
	Mock        string              `json:"mock"`
	Example     string              `json:"example"`
	Children    []ResponseFieldItem `json:"children,omitempty"`
}

type ResponseResultItem struct {
	ID         int64               `json:"id,omitempty"`
	Name       string              `json:"name"`
	StatusCode int                 `json:"status_code"`
	Format     string              `json:"format"`
	DataType   string              `json:"data_type"`
	Fields     []ResponseFieldItem `json:"fields"`
}

type ResponseExampleItem struct {
	ID          int64  `json:"id,omitempty"`
	Name        string `json:"name"`
	StatusCode  int    `json:"status_code"`
	ContentType string `json:"content_type"`
	Raw         string `json:"raw"`
}

type DetailItem struct {
	Item
	RequestHeaders   []ParamItem           `json:"request_headers"`
	RequestBody      RequestBodyConfig     `json:"request_body"`
	QueryParams      []ParamItem           `json:"query_params"`
	ResponseHeaders  []ResponseHeaderItem  `json:"response_headers"`
	ResponseResults  []ResponseResultItem  `json:"response_results"`
	ResponseExamples []ResponseExampleItem `json:"response_examples"`
}

type ListRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
}

type DetailRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	InterfaceID int64 `form:"interface_id" json:"interface_id" binding:"required,min=1"`
}

type CreateRequest struct {
	WorkspaceID      int64                 `json:"workspace_id" binding:"required,min=1"`
	ProjectID        int64                 `json:"project_id" binding:"required,min=1"`
	FolderID         int64                 `json:"folder_id" binding:"required,min=1"`
	Name             string                `json:"name" binding:"required,min=1,max=100"`
	Method           string                `json:"method" binding:"required,min=1,max=10"`
	Protocol         string                `json:"protocol"`
	URL              string                `json:"url" binding:"max=1024"`
	Status           uint8                 `json:"status"`
	RequestHeaders   []ParamItem           `json:"request_headers"`
	RequestBody      RequestBodyConfig     `json:"request_body"`
	QueryParams      []ParamItem           `json:"query_params"`
	ResponseHeaders  []ResponseHeaderItem  `json:"response_headers"`
	ResponseResults  []ResponseResultItem  `json:"response_results"`
	ResponseExamples []ResponseExampleItem `json:"response_examples"`
}

type UpdateRequest struct {
	WorkspaceID      int64                 `json:"workspace_id" binding:"required,min=1"`
	ProjectID        int64                 `json:"project_id" binding:"required,min=1"`
	InterfaceID      int64                 `json:"interface_id" binding:"required,min=1"`
	FolderID         int64                 `json:"folder_id"`
	Name             string                `json:"name" binding:"required,min=1,max=100"`
	Method           string                `json:"method" binding:"required,min=1,max=10"`
	Protocol         string                `json:"protocol"`
	URL              string                `json:"url" binding:"max=1024"`
	Status           uint8                 `json:"status"`
	RequestHeaders   []ParamItem           `json:"request_headers"`
	RequestBody      RequestBodyConfig     `json:"request_body"`
	QueryParams      []ParamItem           `json:"query_params"`
	ResponseHeaders  []ResponseHeaderItem  `json:"response_headers"`
	ResponseResults  []ResponseResultItem  `json:"response_results"`
	ResponseExamples []ResponseExampleItem `json:"response_examples"`
}

type DeleteRequest struct {
	WorkspaceID int64 `form:"workspace_id" json:"workspace_id" binding:"required,min=1"`
	ProjectID   int64 `form:"project_id" json:"project_id" binding:"required,min=1"`
	InterfaceID int64 `form:"interface_id" json:"interface_id" binding:"required,min=1"`
}

type ReorderRequest struct {
	WorkspaceID  int64   `json:"workspace_id" binding:"required,min=1"`
	ProjectID    int64   `json:"project_id" binding:"required,min=1"`
	FolderID     int64   `json:"folder_id" binding:"required,min=1"`
	InterfaceIDs []int64 `json:"interface_ids" binding:"required,min=1,dive,min=1"`
}

func resolveEnabled(v *bool) bool {
	if v == nil {
		return true
	}
	return *v
}

func enabledPtr(v bool) *bool {
	return &v
}
