package importexport

import (
	"encoding/json"
	"strings"
	"time"

	"nest-api/app/interfaces"
	bizerr "nest-api/pkg/errors"
)

type yapiCategory struct {
	Name string    `json:"name"`
	Desc string    `json:"desc"`
	List []yapiAPI `json:"list"`
}

type yapiAPI struct {
	Title               string          `json:"title"`
	Path                string          `json:"path"`
	Method              string          `json:"method"`
	Status              string          `json:"status"`
	Desc                string          `json:"desc"`
	ReqQuery            []yapiParam     `json:"req_query"`
	ReqHeaders          []yapiParam     `json:"req_headers"`
	ReqParams           []yapiParam     `json:"req_params"`
	ReqBodyType         string          `json:"req_body_type"`
	ReqBodyForm         []yapiParam     `json:"req_body_form"`
	ReqBodyOther        string          `json:"req_body_other"`
	ReqBodyIsJSONSchema bool            `json:"req_body_is_json_schema"`
	ResBodyType         string          `json:"res_body_type"`
	ResBody             string          `json:"res_body"`
	ResBodyIsJSONSchema bool            `json:"res_body_is_json_schema"`
	QueryPath           json.RawMessage `json:"query_path"`
}

type yapiParam struct {
	Name     string `json:"name"`
	Value    string `json:"value"`
	Example  string `json:"example"`
	Desc     string `json:"desc"`
	Required any    `json:"required"`
	Type     string `json:"type"`
}

// ConvertYApiDocument converts a YApi category/interface export into ApiNest Document.
func ConvertYApiDocument(raw json.RawMessage) (*Document, error) {
	cats, err := parseYApiCategories(raw)
	if err != nil {
		return nil, err
	}
	if len(cats) == 0 {
		return nil, bizerr.New("YApi file has no categories")
	}

	tree := make([]Node, 0, len(cats))
	apiCount := 0
	for _, cat := range cats {
		folder := convertYApiCategory(cat)
		apiCount += countYApiAPIs(folder)
		tree = append(tree, folder)
	}
	if apiCount == 0 {
		return nil, bizerr.New("YApi file has no importable APIs")
	}

	return &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      ScopeProject,
		Project:    ProjectMeta{},
		Tree:       tree,
	}, nil
}

func parseYApiCategories(raw json.RawMessage) ([]yapiCategory, error) {
	trim := strings.TrimSpace(string(raw))
	if trim == "" {
		return nil, bizerr.New("import file is not valid JSON")
	}

	if strings.HasPrefix(trim, "[") {
		var cats []yapiCategory
		if err := json.Unmarshal(raw, &cats); err != nil {
			return nil, bizerr.New("import file is not valid JSON")
		}
		if !looksLikeYApiCategories(cats) {
			return nil, bizerr.New("not a YApi export file")
		}
		return cats, nil
	}

	var obj map[string]json.RawMessage
	if err := json.Unmarshal(raw, &obj); err != nil {
		return nil, bizerr.New("import file is not valid JSON")
	}

	// Single category object.
	var single yapiCategory
	if err := json.Unmarshal(raw, &single); err == nil && looksLikeYApiCategory(single) {
		return []yapiCategory{single}, nil
	}

	// Wrapped: { data: [...] } or { cats: [...] }
	for _, key := range []string{"data", "cats", "list", "categories"} {
		if payload, ok := obj[key]; ok {
			cats, err := parseYApiCategories(payload)
			if err == nil {
				return cats, nil
			}
		}
	}
	return nil, bizerr.New("not a YApi export file")
}

func looksLikeYApiCategories(cats []yapiCategory) bool {
	if len(cats) == 0 {
		return false
	}
	for _, c := range cats {
		if looksLikeYApiCategory(c) {
			return true
		}
	}
	return false
}

func looksLikeYApiCategory(c yapiCategory) bool {
	if strings.TrimSpace(c.Name) != "" && c.List != nil {
		return true
	}
	for _, api := range c.List {
		if strings.TrimSpace(api.Title) != "" || strings.TrimSpace(api.Path) != "" || strings.TrimSpace(api.Method) != "" {
			return true
		}
	}
	return false
}

func convertYApiCategory(cat yapiCategory) Node {
	children := make([]Node, 0, len(cat.List))
	for _, api := range cat.List {
		children = append(children, convertYApiAPI(api))
	}
	return Node{
		Type:     "folder",
		Name:     fallbackName(cat.Name, "未命名分类"),
		Children: children,
	}
}

func convertYApiAPI(api yapiAPI) Node {
	method := strings.ToUpper(strings.TrimSpace(api.Method))
	if method == "" {
		method = "GET"
	}
	urlStr := strings.TrimSpace(api.Path)
	if urlStr == "" {
		urlStr = yapiQueryPath(api.QueryPath)
	}

	return Node{
		Type:             "api",
		Name:             fallbackName(api.Title, method+" "+urlStr),
		Method:           method,
		URL:              urlStr,
		Status:           mapYApiStatus(api.Status),
		RequestHeaders:   convertYApiParams(api.ReqHeaders, true),
		RequestBody:      convertYApiBody(api),
		QueryParams:      convertYApiParams(api.ReqQuery, false),
		ResponseExamples: convertYApiResponse(api),
	}
}

func convertYApiParams(items []yapiParam, isHeader bool) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Name)
		if key == "" {
			continue
		}
		example := strings.TrimSpace(item.Example)
		if example == "" {
			example = strings.TrimSpace(item.Value)
		}
		enabled := true
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        yapiType(item.Type),
			Required:    yapiRequired(item.Required),
			Enabled:     &enabled,
			Description: item.Desc,
			Example:     example,
		})
		_ = isHeader
	}
	return out
}

func convertYApiBody(api yapiAPI) interfaces.RequestBodyConfig {
	typ := strings.ToLower(strings.TrimSpace(api.ReqBodyType))
	switch typ {
	case "", "none":
		if len(api.ReqBodyForm) == 0 && strings.TrimSpace(api.ReqBodyOther) == "" {
			return emptyRequestBody()
		}
	case "form", "form-data", "file":
		fields := convertYApiBodyFields(api.ReqBodyForm)
		if len(fields) == 0 && api.ReqBodyIsJSONSchema {
			if schema := parseJSONSchema(api.ReqBodyOther); schema != nil {
				fields = schemaToBodyFields(schema, newOpenAPIResolver(map[string]any{}))
			}
		}
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   fields,
		}
	case "json":
		raw := strings.TrimSpace(api.ReqBodyOther)
		if api.ReqBodyIsJSONSchema {
			if schema := parseJSONSchema(raw); schema != nil {
				if schemaHasProperties(schema) {
					return interfaces.RequestBodyConfig{
						Format:   "json",
						DataType: "Object",
						Raw:      "",
						Fields:   schemaToBodyFields(schema, newOpenAPIResolver(map[string]any{})),
					}
				}
				return interfaces.RequestBodyConfig{
					Format:   "raw",
					DataType: "JSON",
					Raw:      schemaExampleJSON(schema, newOpenAPIResolver(map[string]any{})),
					Fields:   []interfaces.BodyFieldItem{},
				}
			}
		}
		if raw == "" {
			return emptyRequestBody()
		}
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "JSON",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	case "raw", "text", "xml":
		dataType := "Text"
		if typ == "xml" {
			dataType = "XML"
		}
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: dataType,
			Raw:      strings.TrimSpace(api.ReqBodyOther),
			Fields:   []interfaces.BodyFieldItem{},
		}
	}

	if len(api.ReqBodyForm) > 0 {
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   convertYApiBodyFields(api.ReqBodyForm),
		}
	}
	if raw := strings.TrimSpace(api.ReqBodyOther); raw != "" {
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "Text",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	}
	return emptyRequestBody()
}

func convertYApiBodyFields(items []yapiParam) []interfaces.BodyFieldItem {
	out := make([]interfaces.BodyFieldItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Name)
		if key == "" {
			continue
		}
		example := strings.TrimSpace(item.Example)
		if example == "" {
			example = strings.TrimSpace(item.Value)
		}
		enabled := true
		out = append(out, interfaces.BodyFieldItem{
			ParentID:    0,
			Name:        key,
			Type:        yapiType(item.Type),
			Required:    yapiRequired(item.Required),
			Enabled:     &enabled,
			Description: item.Desc,
			Example:     example,
			Children:    []interfaces.BodyFieldItem{},
		})
	}
	return out
}

func convertYApiResponse(api yapiAPI) []interfaces.ResponseExampleItem {
	raw := strings.TrimSpace(api.ResBody)
	if raw == "" {
		return nil
	}
	contentType := mapYApiContentType(api.ResBodyType)
	if api.ResBodyIsJSONSchema {
		if schema := parseJSONSchema(raw); schema != nil {
			if example := schemaExampleJSON(schema, newOpenAPIResolver(map[string]any{})); example != "" {
				raw = example
			}
			contentType = "application/json"
		}
	} else if contentType == "" {
		contentType = "application/json"
	}
	return []interfaces.ResponseExampleItem{{
		Name:        "成功",
		StatusCode:  200,
		ContentType: contentType,
		Raw:         raw,
	}}
}

func parseJSONSchema(raw string) map[string]any {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil
	}
	var schema map[string]any
	if err := json.Unmarshal([]byte(raw), &schema); err != nil {
		return nil
	}
	if schema["type"] == nil && schema["properties"] == nil && schema["$schema"] == nil && schema["items"] == nil {
		return nil
	}
	return schema
}

func yapiQueryPath(raw json.RawMessage) string {
	if len(raw) == 0 || string(raw) == "null" {
		return ""
	}
	var qp struct {
		Path string `json:"path"`
	}
	if err := json.Unmarshal(raw, &qp); err != nil {
		return ""
	}
	return strings.TrimSpace(qp.Path)
}

func yapiRequired(v any) bool {
	switch t := v.(type) {
	case bool:
		return t
	case float64:
		return t != 0
	case string:
		s := strings.TrimSpace(t)
		return s == "1" || strings.EqualFold(s, "true")
	default:
		return false
	}
}

func yapiType(t string) string {
	switch strings.ToLower(strings.TrimSpace(t)) {
	case "number", "integer", "int", "float", "double":
		return "number"
	case "boolean", "bool":
		return "boolean"
	case "array":
		return "array"
	case "object":
		return "object"
	case "file":
		return "file"
	default:
		return "string"
	}
}

func mapYApiContentType(ct string) string {
	switch strings.ToLower(strings.TrimSpace(ct)) {
	case "", "json", "application/json":
		return "application/json"
	case "xml", "application/xml", "text/xml":
		return "application/xml"
	case "html", "text/html":
		return "text/html"
	case "text", "plain", "text/plain", "raw":
		return "text/plain"
	default:
		return ct
	}
}

func mapYApiStatus(status string) uint8 {
	switch strings.ToLower(strings.TrimSpace(status)) {
	case "done", "released", "published":
		return 1
	case "testing", "test":
		return 2
	case "deprecated", "obsolete":
		return 4
	default:
		return 3
	}
}

func countYApiAPIs(node Node) int {
	if node.Type == "api" {
		return 1
	}
	n := 0
	for _, child := range node.Children {
		n += countYApiAPIs(child)
	}
	return n
}
