package importexport

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"nest-api/app/interfaces"
	bizerr "nest-api/pkg/errors"
)

type postmanCollection struct {
	Info postmanInfo   `json:"info"`
	Item []postmanItem `json:"item"`
}

type postmanInfo struct {
	Name   string `json:"name"`
	Schema string `json:"schema"`
}

type postmanItem struct {
	Name     string            `json:"name"`
	Item     []postmanItem     `json:"item"`
	Request  json.RawMessage   `json:"request"`
	Response []postmanResponse `json:"response"`
}

type postmanRequest struct {
	Method      string          `json:"method"`
	Header      []postmanKeyVal `json:"header"`
	Body        *postmanBody    `json:"body"`
	URL         json.RawMessage `json:"url"`
	Description json.RawMessage `json:"description"`
}

type postmanBody struct {
	Mode       string          `json:"mode"`
	Raw        string          `json:"raw"`
	URLEncoded []postmanKeyVal `json:"urlencoded"`
	FormData   []postmanKeyVal `json:"formdata"`
	GraphQL    *struct {
		Query     string `json:"query"`
		Variables string `json:"variables"`
	} `json:"graphql"`
	Options *struct {
		Raw *struct {
			Language string `json:"language"`
		} `json:"raw"`
	} `json:"options"`
}

type postmanKeyVal struct {
	Key         string          `json:"key"`
	Value       string          `json:"value"`
	Type        string          `json:"type"`
	Disabled    bool            `json:"disabled"`
	Description json.RawMessage `json:"description"`
	Src         json.RawMessage `json:"src"`
}

type postmanURL struct {
	Raw   string          `json:"raw"`
	Query []postmanKeyVal `json:"query"`
	Host  []string        `json:"host"`
	Path  json.RawMessage `json:"path"`
}

type postmanResponse struct {
	Name                string          `json:"name"`
	Status              string          `json:"status"`
	Code                int             `json:"code"`
	Header              []postmanKeyVal `json:"header"`
	Body                string          `json:"body"`
	PostmanPreviewLang  string          `json:"_postman_previewlanguage"`
}

// ConvertPostmanCollection converts a Postman Collection v2.0/v2.1 document into ApiNest Document.
func ConvertPostmanCollection(raw json.RawMessage) (*Document, error) {
	var col postmanCollection
	if err := json.Unmarshal(raw, &col); err != nil {
		return nil, bizerr.New("import file is not valid JSON")
	}
	if !isPostmanCollection(col) {
		return nil, bizerr.New("not a Postman Collection file")
	}
	if len(col.Item) == 0 {
		return nil, bizerr.New("Postman Collection has no items")
	}

	children := convertPostmanItems(col.Item)
	tree := children
	if name := strings.TrimSpace(col.Info.Name); name != "" {
		tree = []Node{{
			Type:     "folder",
			Name:     name,
			Children: children,
		}}
	}

	return &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      ScopeProject,
		Project:    ProjectMeta{Name: strings.TrimSpace(col.Info.Name)},
		Tree:       tree,
	}, nil
}

func isPostmanCollection(col postmanCollection) bool {
	schema := strings.ToLower(strings.TrimSpace(col.Info.Schema))
	if strings.Contains(schema, "postman") || strings.Contains(schema, "collection") {
		return true
	}
	// Fallback: Postman exports always have info + item, never ApiNest format field.
	return strings.TrimSpace(col.Info.Name) != "" && len(col.Item) > 0
}

func convertPostmanItems(items []postmanItem) []Node {
	nodes := make([]Node, 0, len(items))
	for _, item := range items {
		if len(item.Request) > 0 && string(item.Request) != "null" {
			nodes = append(nodes, convertPostmanRequest(item))
			continue
		}
		if len(item.Item) > 0 {
			nodes = append(nodes, Node{
				Type:     "folder",
				Name:     fallbackName(item.Name, "未命名文件夹"),
				Children: convertPostmanItems(item.Item),
			})
			continue
		}
		// Empty folder
		if strings.TrimSpace(item.Name) != "" {
			nodes = append(nodes, Node{
				Type:     "folder",
				Name:     strings.TrimSpace(item.Name),
				Children: nil,
			})
		}
	}
	return nodes
}

func convertPostmanRequest(item postmanItem) Node {
	var req postmanRequest
	_ = json.Unmarshal(item.Request, &req)

	// request may also be a plain URL string in rare exports
	if req.Method == "" && len(item.Request) > 0 && item.Request[0] == '"' {
		var urlStr string
		if err := json.Unmarshal(item.Request, &urlStr); err == nil {
			req.URL = item.Request
			req.Method = "GET"
		}
	}

	method := strings.ToUpper(strings.TrimSpace(req.Method))
	if method == "" {
		method = "GET"
	}

	urlStr, query := parsePostmanURL(req.URL)
	headers := convertPostmanParams(req.Header, false)
	body := convertPostmanBody(req.Body)

	node := Node{
		Type:           "api",
		Name:           fallbackName(item.Name, "未命名接口"),
		Method:         method,
		URL:            urlStr,
		Status:         3,
		RequestHeaders: headers,
		RequestBody:    body,
		QueryParams:    query,
	}

	if examples := convertPostmanResponses(item.Response); len(examples) > 0 {
		node.ResponseExamples = examples
	}
	return node
}

func parsePostmanURL(raw json.RawMessage) (string, []interfaces.ParamItem) {
	if len(raw) == 0 || string(raw) == "null" {
		return "", nil
	}
	if raw[0] == '"' {
		var s string
		if err := json.Unmarshal(raw, &s); err == nil {
			return s, nil
		}
	}
	var u postmanURL
	if err := json.Unmarshal(raw, &u); err != nil {
		return "", nil
	}
	urlStr := strings.TrimSpace(u.Raw)
	if urlStr == "" {
		urlStr = buildPostmanURL(u)
	}
	return urlStr, convertPostmanParams(u.Query, true)
}

func buildPostmanURL(u postmanURL) string {
	host := strings.Join(u.Host, ".")
	var pathParts []string
	if len(u.Path) > 0 {
		var asStrings []string
		if err := json.Unmarshal(u.Path, &asStrings); err == nil {
			pathParts = asStrings
		}
	}
	path := strings.Join(pathParts, "/")
	switch {
	case host != "" && path != "":
		return host + "/" + path
	case host != "":
		return host
	default:
		return path
	}
}

func convertPostmanParams(items []postmanKeyVal, allowEmptyKey bool) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Key)
		if key == "" && !allowEmptyKey {
			continue
		}
		if key == "" {
			continue
		}
		enabled := !item.Disabled
		typ := "string"
		if strings.EqualFold(strings.TrimSpace(item.Type), "file") {
			typ = "file"
		}
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        typ,
			Required:    false,
			Enabled:     &enabled,
			Description: postmanDescription(item.Description),
			Example:     item.Value,
		})
	}
	return out
}

func convertPostmanBody(body *postmanBody) interfaces.RequestBodyConfig {
	empty := interfaces.RequestBodyConfig{
		Format:   "json",
		DataType: "Object",
		Raw:      "",
		Fields:   []interfaces.BodyFieldItem{},
	}
	if body == nil {
		return empty
	}

	mode := strings.ToLower(strings.TrimSpace(body.Mode))
	switch mode {
	case "raw":
		language := "json"
		if body.Options != nil && body.Options.Raw != nil {
			language = strings.TrimSpace(body.Options.Raw.Language)
		}
		dataType := mapPostmanRawLanguage(language)
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: dataType,
			Raw:      body.Raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	case "urlencoded":
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   convertPostmanBodyFields(body.URLEncoded),
		}
	case "formdata":
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   convertPostmanBodyFields(body.FormData),
		}
	case "graphql":
		raw := ""
		if body.GraphQL != nil {
			raw = body.GraphQL.Query
			if strings.TrimSpace(body.GraphQL.Variables) != "" {
				raw = fmt.Sprintf("%s\n\n# variables\n%s", body.GraphQL.Query, body.GraphQL.Variables)
			}
		}
		if raw == "" {
			raw = body.Raw
		}
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "JSON",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	default:
		if strings.TrimSpace(body.Raw) != "" {
			return interfaces.RequestBodyConfig{
				Format:   "raw",
				DataType: "Text",
				Raw:      body.Raw,
				Fields:   []interfaces.BodyFieldItem{},
			}
		}
		return empty
	}
}

func convertPostmanBodyFields(items []postmanKeyVal) []interfaces.BodyFieldItem {
	fields := make([]interfaces.BodyFieldItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Key)
		if key == "" {
			continue
		}
		enabled := !item.Disabled
		typ := "string"
		if strings.EqualFold(strings.TrimSpace(item.Type), "file") {
			typ = "file"
		}
		fields = append(fields, interfaces.BodyFieldItem{
			ParentID:    0,
			Name:        key,
			Type:        typ,
			Required:    false,
			Enabled:     &enabled,
			Description: postmanDescription(item.Description),
			Example:     item.Value,
			Children:    []interfaces.BodyFieldItem{},
		})
	}
	return fields
}

func convertPostmanResponses(responses []postmanResponse) []interfaces.ResponseExampleItem {
	out := make([]interfaces.ResponseExampleItem, 0, len(responses))
	for i, resp := range responses {
		name := strings.TrimSpace(resp.Name)
		if name == "" {
			name = fmt.Sprintf("示例 %d", i+1)
		}
		statusCode := resp.Code
		if statusCode == 0 {
			statusCode = 200
		}
		contentType := contentTypeFromHeaders(resp.Header)
		if contentType == "" {
			contentType = contentTypeFromPreviewLang(resp.PostmanPreviewLang)
		}
		out = append(out, interfaces.ResponseExampleItem{
			Name:        name,
			StatusCode:  statusCode,
			ContentType: contentType,
			Raw:         resp.Body,
		})
	}
	return out
}

func mapPostmanRawLanguage(language string) string {
	switch strings.ToLower(strings.TrimSpace(language)) {
	case "json", "javascript", "js":
		return "JSON"
	case "xml":
		return "XML"
	case "html":
		return "HTML"
	case "text", "plain":
		return "Text"
	default:
		if language == "" {
			return "JSON"
		}
		return "Text"
	}
}

func contentTypeFromHeaders(headers []postmanKeyVal) string {
	for _, h := range headers {
		if strings.EqualFold(strings.TrimSpace(h.Key), "content-type") {
			return strings.TrimSpace(h.Value)
		}
	}
	return ""
}

func contentTypeFromPreviewLang(lang string) string {
	switch strings.ToLower(strings.TrimSpace(lang)) {
	case "json":
		return "application/json"
	case "xml":
		return "application/xml"
	case "html":
		return "text/html"
	case "text":
		return "text/plain"
	default:
		return "application/json"
	}
}

func postmanDescription(raw json.RawMessage) string {
	if len(raw) == 0 || string(raw) == "null" {
		return ""
	}
	if raw[0] == '"' {
		var s string
		if err := json.Unmarshal(raw, &s); err == nil {
			return s
		}
	}
	var obj struct {
		Content string `json:"content"`
	}
	if err := json.Unmarshal(raw, &obj); err == nil {
		return obj.Content
	}
	return ""
}

func fallbackName(name, fallback string) string {
	name = strings.TrimSpace(name)
	if name == "" {
		return fallback
	}
	return name
}
