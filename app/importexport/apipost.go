package importexport

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"

	"nest-api/app/interfaces"
	bizerr "nest-api/pkg/errors"
)

type apiPostNode struct {
	TargetID   string          `json:"target_id"`
	TargetType string          `json:"target_type"`
	ParentID   string          `json:"parent_id"`
	Name       string          `json:"name"`
	Method     string          `json:"method"`
	URL        string          `json:"url"`
	Request    json.RawMessage `json:"request"`
	Response   json.RawMessage `json:"response"`
	Children   []apiPostNode   `json:"children"`
}

type apiPostRequest struct {
	URL         string          `json:"url"`
	Description string          `json:"description"`
	Header      apiPostParamBox `json:"header"`
	Query       apiPostParamBox `json:"query"`
	Body        apiPostBody     `json:"body"`
	Resful      apiPostParamBox `json:"resful"`
	Restful     apiPostParamBox `json:"restful"`
}

type apiPostParamBox struct {
	Parameter []apiPostParam `json:"parameter"`
}

type apiPostParam struct {
	Key         string `json:"key"`
	Value       string `json:"value"`
	Description string `json:"description"`
	IsChecked   any    `json:"is_checked"`
	NotNull     any    `json:"not_null"`
	FieldType   string `json:"field_type"`
	Type        string `json:"type"`
}

type apiPostBody struct {
	Mode      string         `json:"mode"`
	Raw       string         `json:"raw"`
	Parameter []apiPostParam `json:"parameter"`
}

// ConvertApiPostDocument converts ApiPost native JSON (project object or folder array) into ApiNest Document.
// Only folders and APIs are imported; envs/models/scripts are ignored.
func ConvertApiPostDocument(raw json.RawMessage) (*Document, error) {
	nodes, projectName, err := extractApiPostNodes(raw)
	if err != nil {
		return nil, err
	}
	if len(nodes) == 0 {
		return nil, bizerr.New("ApiPost file has no folders or APIs")
	}

	tree := convertApiPostNodes(nodes)
	if len(tree) == 0 {
		// Flat list with parent_id
		tree = convertApiPostFlat(nodes)
	}
	if len(tree) == 0 {
		return nil, bizerr.New("ApiPost file has no importable APIs")
	}

	return &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      ScopeProject,
		Project:    ProjectMeta{Name: projectName},
		Tree:       tree,
	}, nil
}

func extractApiPostNodes(raw json.RawMessage) ([]apiPostNode, string, error) {
	trimmed := strings.TrimSpace(string(raw))
	if trimmed == "" {
		return nil, "", bizerr.New("import file is not valid JSON")
	}

	switch trimmed[0] {
	case '[':
		var nodes []apiPostNode
		if err := json.Unmarshal(raw, &nodes); err != nil {
			return nil, "", bizerr.New("import file is not valid JSON")
		}
		if !looksLikeApiPostNodes(nodes) {
			return nil, "", bizerr.New("not an ApiPost export file")
		}
		return nodes, "", nil
	case '{':
		var wrapper struct {
			Project *struct {
				Name string `json:"name"`
			} `json:"project"`
			Apis []apiPostNode `json:"apis"`
		}
		if err := json.Unmarshal(raw, &wrapper); err != nil {
			return nil, "", bizerr.New("import file is not valid JSON")
		}
		if len(wrapper.Apis) == 0 || !looksLikeApiPostNodes(wrapper.Apis) {
			// Sometimes root itself is a single folder/api object.
			var one apiPostNode
			if err := json.Unmarshal(raw, &one); err == nil && strings.TrimSpace(one.TargetType) != "" {
				return []apiPostNode{one}, strings.TrimSpace(one.Name), nil
			}
			return nil, "", bizerr.New("not an ApiPost export file")
		}
		name := ""
		if wrapper.Project != nil {
			name = strings.TrimSpace(wrapper.Project.Name)
		}
		return wrapper.Apis, name, nil
	default:
		return nil, "", bizerr.New("import file is not valid JSON")
	}
}

func looksLikeApiPostNodes(nodes []apiPostNode) bool {
	for _, n := range nodes {
		t := strings.ToLower(strings.TrimSpace(n.TargetType))
		if t == "api" || t == "folder" || t == "dir" || t == "group" {
			return true
		}
		if len(n.Children) > 0 || len(n.Request) > 0 {
			return true
		}
	}
	return false
}

func convertApiPostNodes(nodes []apiPostNode) []Node {
	out := make([]Node, 0, len(nodes))
	hasNested := false
	for _, n := range nodes {
		if len(n.Children) > 0 {
			hasNested = true
			break
		}
	}
	if !hasNested {
		// Keep empty; caller may rebuild from parent_id.
		for _, n := range nodes {
			if strings.TrimSpace(n.ParentID) != "" && n.ParentID != "0" {
				return nil
			}
		}
	}

	for _, n := range nodes {
		if node, ok := convertApiPostNode(n); ok {
			out = append(out, node)
		}
	}
	return out
}

func convertApiPostFlat(nodes []apiPostNode) []Node {
	byParent := map[string][]apiPostNode{}
	ids := map[string]struct{}{}
	for _, n := range nodes {
		id := strings.TrimSpace(n.TargetID)
		if id != "" {
			ids[id] = struct{}{}
		}
	}
	for _, n := range nodes {
		parent := strings.TrimSpace(n.ParentID)
		if parent == "" || parent == "0" || parent == "1" {
			parent = "0"
		} else if _, ok := ids[parent]; !ok {
			parent = "0"
		}
		byParent[parent] = append(byParent[parent], n)
	}

	var build func(parent string) []Node
	build = func(parent string) []Node {
		list := byParent[parent]
		out := make([]Node, 0, len(list))
		for _, n := range list {
			kind := strings.ToLower(strings.TrimSpace(n.TargetType))
			switch kind {
			case "folder", "dir", "group":
				children := build(strings.TrimSpace(n.TargetID))
				// Also attach nested children field if present.
				if len(n.Children) > 0 {
					children = append(children, convertApiPostNodes(n.Children)...)
				}
				out = append(out, Node{
					Type:     "folder",
					Name:     fallbackName(n.Name, "未命名文件夹"),
					Children: children,
				})
			case "api", "":
				if kind == "" && len(n.Request) == 0 && n.Method == "" {
					continue
				}
				if node, ok := convertApiPostAPI(n); ok {
					out = append(out, node)
				}
			}
		}
		return out
	}
	return build("0")
}

func convertApiPostNode(n apiPostNode) (Node, bool) {
	kind := strings.ToLower(strings.TrimSpace(n.TargetType))
	switch kind {
	case "folder", "dir", "group":
		children := convertApiPostNodes(n.Children)
		return Node{
			Type:     "folder",
			Name:     fallbackName(n.Name, "未命名文件夹"),
			Children: children,
		}, true
	case "api", "":
		if kind == "" && len(n.Request) == 0 && strings.TrimSpace(n.Method) == "" {
			return Node{}, false
		}
		return convertApiPostAPI(n)
	default:
		// Skip websocket/socket/docs/etc.
		return Node{}, false
	}
}

func convertApiPostAPI(n apiPostNode) (Node, bool) {
	var req apiPostRequest
	if len(n.Request) > 0 && string(n.Request) != "null" {
		_ = json.Unmarshal(n.Request, &req)
	}

	method := strings.ToUpper(strings.TrimSpace(n.Method))
	if method == "" {
		method = "GET"
	}
	urlStr := strings.TrimSpace(n.URL)
	if urlStr == "" {
		urlStr = strings.TrimSpace(req.URL)
	}

	headers := convertApiPostParams(req.Header.Parameter)
	query := convertApiPostParams(req.Query.Parameter)
	body := convertApiPostBody(req.Body)
	examples := convertApiPostResponses(n.Response)

	return Node{
		Type:             "api",
		Name:             fallbackName(n.Name, "未命名接口"),
		Method:           method,
		URL:              urlStr,
		Status:           3,
		RequestHeaders:   headers,
		RequestBody:      body,
		QueryParams:      query,
		ResponseExamples: examples,
	}, true
}

func convertApiPostParams(items []apiPostParam) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.Key)
		if key == "" {
			continue
		}
		enabled := apiPostChecked(item.IsChecked)
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        apiPostFieldType(item),
			Required:    apiPostTruthy(item.NotNull),
			Enabled:     &enabled,
			Description: item.Description,
			Example:     item.Value,
		})
	}
	return out
}

func convertApiPostBody(body apiPostBody) interfaces.RequestBodyConfig {
	mode := strings.ToLower(strings.TrimSpace(body.Mode))
	switch mode {
	case "form-data", "formdata", "urlencoded", "x-www-form-urlencoded":
		fields := make([]interfaces.BodyFieldItem, 0, len(body.Parameter))
		for _, item := range body.Parameter {
			key := strings.TrimSpace(item.Key)
			if key == "" {
				continue
			}
			enabled := apiPostChecked(item.IsChecked)
			fields = append(fields, interfaces.BodyFieldItem{
				ParentID:    0,
				Name:        key,
				Type:        apiPostFieldType(item),
				Required:    apiPostTruthy(item.NotNull),
				Enabled:     &enabled,
				Description: item.Description,
				Example:     item.Value,
				Children:    []interfaces.BodyFieldItem{},
			})
		}
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   fields,
		}
	case "json", "raw", "javascript", "xml", "text", "html", "msgpack":
		dataType := "JSON"
		switch mode {
		case "xml":
			dataType = "XML"
		case "html":
			dataType = "HTML"
		case "text":
			dataType = "Text"
		case "javascript":
			dataType = "JSON"
		}
		raw := body.Raw
		if raw == "" && len(body.Parameter) > 0 {
			// Some exports keep json fields only in parameter list.
			obj := map[string]any{}
			for _, item := range body.Parameter {
				if strings.TrimSpace(item.Key) == "" {
					continue
				}
				obj[item.Key] = item.Value
			}
			if b, err := json.MarshalIndent(obj, "", "  "); err == nil {
				raw = string(b)
			}
		}
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: dataType,
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	default:
		if strings.TrimSpace(body.Raw) != "" {
			return interfaces.RequestBodyConfig{
				Format:   "raw",
				DataType: "JSON",
				Raw:      body.Raw,
				Fields:   []interfaces.BodyFieldItem{},
			}
		}
		return interfaces.RequestBodyConfig{
			Format:   "json",
			DataType: "Object",
			Raw:      "",
			Fields:   []interfaces.BodyFieldItem{},
		}
	}
}

func convertApiPostResponses(raw json.RawMessage) []interfaces.ResponseExampleItem {
	if len(raw) == 0 || string(raw) == "null" {
		return nil
	}

	// Newer exports may use array.
	var asArray []map[string]any
	if err := json.Unmarshal(raw, &asArray); err == nil && len(asArray) > 0 {
		out := make([]interfaces.ResponseExampleItem, 0, len(asArray))
		for i, item := range asArray {
			name := strings.TrimSpace(stringValue(item["name"]))
			if name == "" {
				name = fmt.Sprintf("示例 %d", i+1)
			}
			code := 200
			switch v := item["code"].(type) {
			case float64:
				code = int(v)
			case string:
				if n, err := strconv.Atoi(v); err == nil {
					code = n
				}
			}
			contentType := "application/json"
			if ct := strings.TrimSpace(stringValue(item["contentType"])); ct != "" {
				contentType = mapApiPostContentType(ct)
			}
			rawBody := strings.TrimSpace(stringValue(item["raw"]))
			if rawBody == "" {
				rawBody = strings.TrimSpace(stringValue(item["mock"]))
			}
			if expect, ok := item["expect"].(map[string]any); ok {
				if rawBody == "" {
					rawBody = strings.TrimSpace(stringValue(expect["mock"]))
				}
				if name == fmt.Sprintf("示例 %d", i+1) {
					if n := strings.TrimSpace(stringValue(expect["name"])); n != "" {
						name = n
					}
				}
				if c, ok := expect["code"].(float64); ok {
					code = int(c)
				}
			}
			if rawBody == "" {
				continue
			}
			out = append(out, interfaces.ResponseExampleItem{
				Name:        name,
				StatusCode:  code,
				ContentType: contentType,
				Raw:         rawBody,
			})
		}
		return out
	}

	var asMap map[string]json.RawMessage
	if err := json.Unmarshal(raw, &asMap); err != nil {
		return nil
	}
	out := make([]interfaces.ResponseExampleItem, 0, len(asMap))
	for key, value := range asMap {
		var item map[string]any
		if err := json.Unmarshal(value, &item); err != nil {
			continue
		}
		expect, _ := item["expect"].(map[string]any)
		name := strings.TrimSpace(stringValue(item["name"]))
		if name == "" && expect != nil {
			name = strings.TrimSpace(stringValue(expect["name"]))
		}
		if name == "" {
			switch key {
			case "success":
				name = "成功"
			case "error":
				name = "失败"
			default:
				name = key
			}
		}
		code := 200
		if expect != nil {
			if c, ok := expect["code"].(float64); ok {
				code = int(c)
			}
		}
		contentType := "application/json"
		if expect != nil {
			if ct := strings.TrimSpace(stringValue(expect["contentType"])); ct != "" {
				contentType = mapApiPostContentType(ct)
			}
		}
		rawBody := strings.TrimSpace(stringValue(item["raw"]))
		if rawBody == "" && expect != nil {
			rawBody = strings.TrimSpace(stringValue(expect["mock"]))
		}
		if rawBody == "" {
			continue
		}
		out = append(out, interfaces.ResponseExampleItem{
			Name:        name,
			StatusCode:  code,
			ContentType: contentType,
			Raw:         rawBody,
		})
	}
	return out
}

func mapApiPostContentType(ct string) string {
	switch strings.ToLower(strings.TrimSpace(ct)) {
	case "json", "application/json":
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

func apiPostFieldType(item apiPostParam) string {
	t := strings.ToLower(strings.TrimSpace(item.FieldType))
	if t == "" {
		t = strings.ToLower(strings.TrimSpace(item.Type))
	}
	switch t {
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

func apiPostChecked(v any) bool {
	switch t := v.(type) {
	case nil:
		return true
	case bool:
		return t
	case float64:
		return t != 0
	case string:
		s := strings.TrimSpace(t)
		return s != "" && s != "0" && !strings.EqualFold(s, "false")
	default:
		return true
	}
}

func apiPostTruthy(v any) bool {
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
