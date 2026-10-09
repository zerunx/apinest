package importexport

import (
	"encoding/json"
	"strings"
	"time"

	"nest-api/app/interfaces"
	bizerr "nest-api/pkg/errors"
)

type apifoxRoot struct {
	ApifoxProject  string          `json:"apifoxProject"`
	Schema         json.RawMessage `json:"$schema"`
	Info           apifoxInfo      `json:"info"`
	APICollection  []apifoxItem    `json:"apiCollection"`
}

type apifoxInfo struct {
	Name string `json:"name"`
}

type apifoxItem struct {
	Name  string          `json:"name"`
	Items []apifoxItem    `json:"items"`
	API   json.RawMessage `json:"api"`
}

type apifoxAPI struct {
	Method            string            `json:"method"`
	Path              string            `json:"path"`
	Parameters        apifoxParams      `json:"parameters"`
	CommonParameters  apifoxParams      `json:"commonParameters"`
	RequestBody       apifoxRequestBody `json:"requestBody"`
	Responses         []apifoxResponse  `json:"responses"`
	ResponseExamples  []apifoxRespEx    `json:"responseExamples"`
	Description       string            `json:"description"`
	Status            string            `json:"status"`
}

type apifoxParams struct {
	Path   []apifoxParam `json:"path"`
	Query  []apifoxParam `json:"query"`
	Header []apifoxParam `json:"header"`
	Cookie []apifoxParam `json:"cookie"`
	Body   []apifoxParam `json:"body"`
}

type apifoxParam struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Required    bool   `json:"required"`
	Example     any    `json:"example"`
	Enable      *bool  `json:"enable"`
	Type        string `json:"type"`
}

type apifoxRequestBody struct {
	Type       string         `json:"type"`
	Parameters []apifoxParam  `json:"parameters"`
	MediaType  string         `json:"mediaType"`
	Examples   []apifoxRespEx `json:"examples"`
	JSONSchema json.RawMessage `json:"jsonSchema"`
}

type apifoxResponse struct {
	ID          any             `json:"id"`
	Code        any             `json:"code"`
	Name        string          `json:"name"`
	ContentType string          `json:"contentType"`
	JSONSchema  json.RawMessage `json:"jsonSchema"`
}

type apifoxRespEx struct {
	Name       string `json:"name"`
	Data       string `json:"data"`
	ResponseID any    `json:"responseId"`
	Value      any    `json:"value"`
}

// ConvertApifoxDocument converts an Apifox project export into ApiNest Document.
// Only apiCollection folders/APIs are imported.
func ConvertApifoxDocument(raw json.RawMessage) (*Document, error) {
	var root apifoxRoot
	if err := json.Unmarshal(raw, &root); err != nil {
		return nil, bizerr.New("import file is not valid JSON")
	}
	if !isApifoxDocument(root) {
		return nil, bizerr.New("not an Apifox export file")
	}
	if len(root.APICollection) == 0 {
		return nil, bizerr.New("Apifox file has no API collection")
	}

	children := convertApifoxItems(root.APICollection)
	if len(children) == 0 {
		return nil, bizerr.New("Apifox file has no importable APIs")
	}

	name := strings.TrimSpace(root.Info.Name)
	tree := children
	// Skip meaningless single "根目录" wrapper if it only contains folders/apis.
	if len(children) == 1 && children[0].Type == "folder" && children[0].Name == "根目录" {
		tree = children[0].Children
		if len(tree) == 0 {
			tree = children
		}
	}
	if name != "" {
		tree = []Node{{
			Type:     "folder",
			Name:     name,
			Children: tree,
		}}
	}

	return &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      ScopeProject,
		Project:    ProjectMeta{Name: name},
		Tree:       tree,
	}, nil
}

func isApifoxDocument(root apifoxRoot) bool {
	if strings.TrimSpace(root.ApifoxProject) != "" {
		return true
	}
	schema := strings.ToLower(string(root.Schema))
	if strings.Contains(schema, "apifox") {
		return true
	}
	return len(root.APICollection) > 0 && strings.TrimSpace(root.Info.Name) != ""
}

func convertApifoxItems(items []apifoxItem) []Node {
	out := make([]Node, 0, len(items))
	for _, item := range items {
		if len(item.API) > 0 && string(item.API) != "null" {
			var api apifoxAPI
			if err := json.Unmarshal(item.API, &api); err != nil {
				continue
			}
			out = append(out, convertApifoxAPI(item.Name, api))
			continue
		}
		if len(item.Items) > 0 || strings.TrimSpace(item.Name) != "" {
			out = append(out, Node{
				Type:     "folder",
				Name:     fallbackName(item.Name, "未命名文件夹"),
				Children: convertApifoxItems(item.Items),
			})
		}
	}
	return out
}

func convertApifoxAPI(name string, api apifoxAPI) Node {
	method := strings.ToUpper(strings.TrimSpace(api.Method))
	if method == "" {
		method = "GET"
	}
	urlStr := strings.TrimSpace(api.Path)

	headers := convertApifoxParams(append(api.CommonParameters.Header, api.Parameters.Header...))
	query := convertApifoxParams(append(api.CommonParameters.Query, api.Parameters.Query...))
	body := convertApifoxBody(api.RequestBody)
	examples := convertApifoxExamples(api.Responses, api.ResponseExamples)

	return Node{
		Type:             "api",
		Name:             fallbackName(name, method+" "+urlStr),
		Method:           method,
		URL:              urlStr,
		Status:           mapApifoxStatus(api.Status),
		RequestHeaders:   headers,
		RequestBody:      body,
		QueryParams:      query,
		ResponseExamples: examples,
	}
}

func convertApifoxParams(items []apifoxParam) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Name)
		if key == "" {
			continue
		}
		enabled := true
		if item.Enable != nil {
			enabled = *item.Enable
		}
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        apifoxType(item.Type),
			Required:    item.Required,
			Enabled:     &enabled,
			Description: item.Description,
			Example:     openAPIExampleString(item.Example),
		})
	}
	return out
}

func convertApifoxBody(body apifoxRequestBody) interfaces.RequestBodyConfig {
	typ := strings.ToLower(strings.TrimSpace(body.Type))
	switch {
	case typ == "" || typ == "none":
		return emptyRequestBody()
	case strings.Contains(typ, "form-urlencoded"), strings.Contains(typ, "multipart"), typ == "form-data":
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   convertApifoxBodyFields(body.Parameters),
		}
	case strings.Contains(typ, "json"):
		if raw := firstApifoxExampleRaw(body.Examples); raw != "" {
			return interfaces.RequestBodyConfig{
				Format:   "raw",
				DataType: "JSON",
				Raw:      raw,
				Fields:   []interfaces.BodyFieldItem{},
			}
		}
		if len(body.Parameters) > 0 {
			return interfaces.RequestBodyConfig{
				Format:   "json",
				DataType: "Object",
				Raw:      "",
				Fields:   convertApifoxBodyFields(body.Parameters),
			}
		}
		if len(body.JSONSchema) > 0 && string(body.JSONSchema) != "null" {
			var schema map[string]any
			if err := json.Unmarshal(body.JSONSchema, &schema); err == nil {
				return interfaces.RequestBodyConfig{
					Format:   "json",
					DataType: "Object",
					Raw:      "",
					Fields:   schemaToBodyFields(schema, newOpenAPIResolver(map[string]any{})),
				}
			}
		}
		return emptyRequestBody()
	case strings.Contains(typ, "xml"):
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "XML",
			Raw:      firstApifoxExampleRaw(body.Examples),
			Fields:   []interfaces.BodyFieldItem{},
		}
	default:
		raw := firstApifoxExampleRaw(body.Examples)
		if len(body.Parameters) > 0 && raw == "" {
			return interfaces.RequestBodyConfig{
				Format:   "form-data",
				DataType: "Object",
				Raw:      "",
				Fields:   convertApifoxBodyFields(body.Parameters),
			}
		}
		if raw == "" {
			return emptyRequestBody()
		}
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "Text",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	}
}

func convertApifoxBodyFields(items []apifoxParam) []interfaces.BodyFieldItem {
	out := make([]interfaces.BodyFieldItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Name)
		if key == "" {
			continue
		}
		enabled := true
		if item.Enable != nil {
			enabled = *item.Enable
		}
		out = append(out, interfaces.BodyFieldItem{
			ParentID:    0,
			Name:        key,
			Type:        apifoxType(item.Type),
			Required:    item.Required,
			Enabled:     &enabled,
			Description: item.Description,
			Example:     openAPIExampleString(item.Example),
			Children:    []interfaces.BodyFieldItem{},
		})
	}
	return out
}

func convertApifoxExamples(responses []apifoxResponse, examples []apifoxRespEx) []interfaces.ResponseExampleItem {
	codeByID := map[string]int{}
	nameByID := map[string]string{}
	contentByID := map[string]string{}
	for _, resp := range responses {
		id := stringValue(resp.ID)
		codeByID[id] = eoInt(resp.Code)
		nameByID[id] = strings.TrimSpace(resp.Name)
		contentByID[id] = mapApifoxContentType(resp.ContentType)
	}

	out := make([]interfaces.ResponseExampleItem, 0, len(examples)+len(responses))
	for _, ex := range examples {
		raw := strings.TrimSpace(ex.Data)
		if raw == "" && ex.Value != nil {
			raw = prettyJSON(ex.Value)
		}
		if raw == "" {
			continue
		}
		id := stringValue(ex.ResponseID)
		code := codeByID[id]
		if code == 0 {
			code = 200
		}
		name := strings.TrimSpace(ex.Name)
		if name == "" {
			name = nameByID[id]
		}
		if name == "" {
			name = "示例"
		}
		contentType := contentByID[id]
		if contentType == "" {
			contentType = "application/json"
		}
		out = append(out, interfaces.ResponseExampleItem{
			Name:        name,
			StatusCode:  code,
			ContentType: contentType,
			Raw:         raw,
		})
	}

	if len(out) == 0 {
		for _, resp := range responses {
			code := eoInt(resp.Code)
			if code == 0 {
				code = 200
			}
			name := strings.TrimSpace(resp.Name)
			if name == "" {
				name = "响应"
			}
			raw := ""
			if len(resp.JSONSchema) > 0 && string(resp.JSONSchema) != "null" {
				var schema map[string]any
				if err := json.Unmarshal(resp.JSONSchema, &schema); err == nil {
					raw = schemaExampleJSON(schema, newOpenAPIResolver(map[string]any{}))
				}
			}
			if raw == "" {
				continue
			}
			out = append(out, interfaces.ResponseExampleItem{
				Name:        name,
				StatusCode:  code,
				ContentType: mapApifoxContentType(resp.ContentType),
				Raw:         raw,
			})
		}
	}
	return out
}

func firstApifoxExampleRaw(examples []apifoxRespEx) string {
	for _, ex := range examples {
		if s := strings.TrimSpace(ex.Data); s != "" {
			return s
		}
		if ex.Value != nil {
			return prettyJSON(ex.Value)
		}
	}
	return ""
}

func apifoxType(t string) string {
	switch strings.ToLower(strings.TrimSpace(t)) {
	case "integer", "number", "int", "float", "double":
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

func mapApifoxContentType(ct string) string {
	switch strings.ToLower(strings.TrimSpace(ct)) {
	case "", "json", "application/json":
		return "application/json"
	case "xml", "application/xml":
		return "application/xml"
	case "html", "text/html":
		return "text/html"
	case "text", "plain", "text/plain":
		return "text/plain"
	default:
		return ct
	}
}

func mapApifoxStatus(status string) uint8 {
	switch strings.ToLower(strings.TrimSpace(status)) {
	case "released", "done", "published":
		return 1
	case "testing", "test":
		return 2
	case "deprecated", "obsolete":
		return 4
	default:
		return 3
	}
}
