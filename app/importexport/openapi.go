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

type openAPIRoot struct {
	OpenAPI     string                     `json:"openapi"`
	Swagger     string                     `json:"swagger"`
	Info        openAPIInfo                `json:"info"`
	Paths       map[string]json.RawMessage `json:"paths"`
	Components  json.RawMessage            `json:"components"`
	Definitions json.RawMessage            `json:"definitions"`
}

type openAPIInfo struct {
	Title       string `json:"title"`
	Description string `json:"description"`
	Version     string `json:"version"`
}

type openAPIOperation struct {
	Tags        []string          `json:"tags"`
	Summary     string            `json:"summary"`
	OperationID string            `json:"operationId"`
	Description string            `json:"description"`
	Parameters  []json.RawMessage `json:"parameters"`
	RequestBody json.RawMessage   `json:"requestBody"`
	Responses   json.RawMessage   `json:"responses"`
	Deprecated  bool              `json:"deprecated"`
}

var openAPIMethods = []string{"get", "post", "put", "patch", "delete", "head", "options", "trace"}

// ConvertOpenAPIDocument converts OpenAPI 3.x / Swagger 2.0 JSON into ApiNest Document.
func ConvertOpenAPIDocument(raw json.RawMessage) (*Document, error) {
	var root openAPIRoot
	if err := json.Unmarshal(raw, &root); err != nil {
		return nil, bizerr.New("import file is not valid JSON")
	}
	if !isOpenAPIDocument(root) {
		return nil, bizerr.New("not an OpenAPI / Swagger file")
	}
	if len(root.Paths) == 0 {
		return nil, bizerr.New("OpenAPI document has no paths")
	}

	var rootMap map[string]any
	_ = json.Unmarshal(raw, &rootMap)
	resolver := newOpenAPIResolver(rootMap)

	folders := map[string][]Node{}
	folderOrder := make([]string, 0)
	var untagged []Node

	for path, pathRaw := range root.Paths {
		if strings.HasPrefix(path, "x-") {
			continue
		}
		var pathItem map[string]json.RawMessage
		if err := json.Unmarshal(pathRaw, &pathItem); err != nil {
			continue
		}
		sharedParams := pathItem["parameters"]
		for _, method := range openAPIMethods {
			opRaw, ok := pathItem[method]
			if !ok || len(opRaw) == 0 || string(opRaw) == "null" {
				continue
			}
			var op openAPIOperation
			if err := json.Unmarshal(opRaw, &op); err != nil {
				continue
			}
			node := convertOpenAPIOperation(path, strings.ToUpper(method), op, sharedParams, resolver)
			tag := ""
			if len(op.Tags) > 0 {
				tag = strings.TrimSpace(op.Tags[0])
			}
			if tag == "" {
				untagged = append(untagged, node)
				continue
			}
			if _, exists := folders[tag]; !exists {
				folderOrder = append(folderOrder, tag)
			}
			folders[tag] = append(folders[tag], node)
		}
	}

	children := make([]Node, 0, len(folderOrder)+len(untagged))
	for _, tag := range folderOrder {
		children = append(children, Node{
			Type:     "folder",
			Name:     tag,
			Children: folders[tag],
		})
	}
	children = append(children, untagged...)
	if len(children) == 0 {
		return nil, bizerr.New("OpenAPI document has no operations")
	}

	title := strings.TrimSpace(root.Info.Title)
	tree := children
	if title != "" {
		tree = []Node{{
			Type:     "folder",
			Name:     title,
			Children: children,
		}}
	}

	return &Document{
		Format:     FormatApinest,
		Version:    Version,
		ExportedAt: time.Now().UTC().Format(time.RFC3339),
		Scope:      ScopeProject,
		Project:    ProjectMeta{Name: title},
		Tree:       tree,
	}, nil
}

func isOpenAPIDocument(root openAPIRoot) bool {
	if strings.TrimSpace(root.OpenAPI) != "" {
		return true
	}
	if strings.TrimSpace(root.Swagger) != "" {
		return true
	}
	return len(root.Paths) > 0 && (strings.TrimSpace(root.Info.Title) != "" || strings.TrimSpace(root.Info.Version) != "")
}

func convertOpenAPIOperation(
	path, method string,
	op openAPIOperation,
	sharedParams json.RawMessage,
	resolver *openAPIResolver,
) Node {
	name := strings.TrimSpace(op.Summary)
	if name == "" {
		name = strings.TrimSpace(op.OperationID)
	}
	if name == "" {
		name = method + " " + path
	}

	headers, query, bodyFromParams := convertOpenAPIParameters(op.Parameters, sharedParams, resolver)
	body := bodyFromParams
	if len(op.RequestBody) > 0 && string(op.RequestBody) != "null" {
		if converted := convertOpenAPIRequestBody(op.RequestBody, resolver); converted != nil {
			body = *converted
		}
	}
	examples := convertOpenAPIResponses(op.Responses, resolver)

	return Node{
		Type:             "api",
		Name:             name,
		Method:           method,
		URL:              path,
		Status:           3,
		RequestHeaders:   headers,
		RequestBody:      body,
		QueryParams:      query,
		ResponseExamples: examples,
	}
}

func convertOpenAPIParameters(
	opParams []json.RawMessage,
	sharedParams json.RawMessage,
	resolver *openAPIResolver,
) (headers, query []interfaces.ParamItem, body interfaces.RequestBodyConfig) {
	body = emptyRequestBody()
	all := make([]json.RawMessage, 0, len(opParams)+8)
	if len(sharedParams) > 0 && string(sharedParams) != "null" {
		var shared []json.RawMessage
		if err := json.Unmarshal(sharedParams, &shared); err == nil {
			all = append(all, shared...)
		}
	}
	all = append(all, opParams...)

	for _, raw := range all {
		param := resolver.resolve(raw)
		in := strings.ToLower(stringValue(param["in"]))
		name := strings.TrimSpace(stringValue(param["name"]))
		if name == "" && in != "body" {
			continue
		}
		required, _ := param["required"].(bool)
		desc := stringValue(param["description"])
		example := openAPIExampleString(param["example"])
		if example == "" {
			if schema, ok := param["schema"].(map[string]any); ok {
				example = openAPIExampleString(schema["example"])
				if example == "" {
					example = openAPIExampleString(schema["default"])
				}
			}
		}
		enabled := true
		item := interfaces.ParamItem{
			Name:        name,
			Type:        openAPIParamType(param),
			Required:    required,
			Enabled:     &enabled,
			Description: desc,
			Example:     example,
		}
		switch in {
		case "header":
			headers = append(headers, item)
		case "query":
			query = append(query, item)
		case "body":
			// Swagger 2.0 body parameter
			if schema, ok := param["schema"].(map[string]any); ok {
				body = schemaToRequestBody(schema, resolver)
			}
		case "formData":
			fields := body.Fields
			if body.Format != "form-data" {
				fields = nil
			}
			typ := "string"
			if strings.EqualFold(stringValue(param["type"]), "file") {
				typ = "file"
			}
			fields = append(fields, interfaces.BodyFieldItem{
				ParentID:    0,
				Name:        name,
				Type:        typ,
				Required:    required,
				Enabled:     &enabled,
				Description: desc,
				Example:     example,
				Children:    []interfaces.BodyFieldItem{},
			})
			body = interfaces.RequestBodyConfig{
				Format:   "form-data",
				DataType: "Object",
				Raw:      "",
				Fields:   fields,
			}
		}
	}
	return headers, query, body
}

func convertOpenAPIRequestBody(raw json.RawMessage, resolver *openAPIResolver) *interfaces.RequestBodyConfig {
	obj := resolver.resolve(raw)
	content, _ := obj["content"].(map[string]any)
	if len(content) == 0 {
		return nil
	}

	prefer := []string{
		"application/json",
		"application/xml",
		"text/plain",
		"multipart/form-data",
		"application/x-www-form-urlencoded",
	}
	var mediaType string
	var media map[string]any
	for _, key := range prefer {
		if v, ok := content[key].(map[string]any); ok {
			mediaType = key
			media = v
			break
		}
	}
	if media == nil {
		for key, v := range content {
			if m, ok := v.(map[string]any); ok {
				mediaType = key
				media = m
				break
			}
		}
	}
	if media == nil {
		return nil
	}

	schema, _ := media["schema"].(map[string]any)
	schema = resolver.resolveMap(schema)

	switch {
	case strings.Contains(mediaType, "multipart/form-data"),
		strings.Contains(mediaType, "application/x-www-form-urlencoded"):
		fields := schemaToBodyFields(schema, resolver)
		return &interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   fields,
		}
	default:
		rawText := ""
		if ex := media["example"]; ex != nil {
			rawText = prettyJSON(ex)
		} else if examples, ok := media["examples"].(map[string]any); ok {
			for _, ex := range examples {
				if m, ok := ex.(map[string]any); ok {
					if v, ok := m["value"]; ok {
						rawText = prettyJSON(v)
						break
					}
				}
			}
		}
		if rawText == "" && schema != nil {
			if ex, ok := schema["example"]; ok {
				rawText = prettyJSON(ex)
			}
		}
		dataType := "JSON"
		format := "raw"
		if strings.Contains(mediaType, "xml") {
			dataType = "XML"
		} else if strings.Contains(mediaType, "text/plain") {
			dataType = "Text"
		}
		// Prefer structured fields when schema has properties and no example raw.
		if rawText == "" && schemaHasProperties(schema) {
			return &interfaces.RequestBodyConfig{
				Format:   "json",
				DataType: "Object",
				Raw:      "",
				Fields:   schemaToBodyFields(schema, resolver),
			}
		}
		if rawText == "" {
			rawText = schemaExampleJSON(schema, resolver)
		}
		return &interfaces.RequestBodyConfig{
			Format:   format,
			DataType: dataType,
			Raw:      rawText,
			Fields:   []interfaces.BodyFieldItem{},
		}
	}
}

func convertOpenAPIResponses(raw json.RawMessage, resolver *openAPIResolver) []interfaces.ResponseExampleItem {
	if len(raw) == 0 || string(raw) == "null" {
		return nil
	}
	var responses map[string]json.RawMessage
	if err := json.Unmarshal(raw, &responses); err != nil {
		return nil
	}
	out := make([]interfaces.ResponseExampleItem, 0)
	for code, respRaw := range responses {
		if strings.HasPrefix(code, "x-") {
			continue
		}
		obj := resolver.resolve(respRaw)
		name := strings.TrimSpace(stringValue(obj["description"]))
		if name == "" {
			name = code
		}
		statusCode := 200
		if code != "default" {
			if n, err := strconv.Atoi(code); err == nil {
				statusCode = n
			}
		}
		contentType := "application/json"
		rawBody := ""
		if content, ok := obj["content"].(map[string]any); ok {
			for mt, mediaVal := range content {
				media, _ := mediaVal.(map[string]any)
				contentType = mt
				if media != nil {
					if ex, ok := media["example"]; ok {
						rawBody = prettyJSON(ex)
					} else if schema, ok := media["schema"].(map[string]any); ok {
						schema = resolver.resolveMap(schema)
						if ex, ok := schema["example"]; ok {
							rawBody = prettyJSON(ex)
						} else {
							rawBody = schemaExampleJSON(schema, resolver)
						}
					}
				}
				break
			}
		} else if schema, ok := obj["schema"].(map[string]any); ok {
			// Swagger 2.0
			schema = resolver.resolveMap(schema)
			if ex, ok := schema["example"]; ok {
				rawBody = prettyJSON(ex)
			} else {
				rawBody = schemaExampleJSON(schema, resolver)
			}
		}
		if rawBody == "" && name == code {
			continue
		}
		out = append(out, interfaces.ResponseExampleItem{
			Name:        name,
			StatusCode:  statusCode,
			ContentType: contentType,
			Raw:         rawBody,
		})
	}
	return out
}

func schemaToRequestBody(schema map[string]any, resolver *openAPIResolver) interfaces.RequestBodyConfig {
	schema = resolver.resolveMap(schema)
	if schemaHasProperties(schema) {
		return interfaces.RequestBodyConfig{
			Format:   "json",
			DataType: "Object",
			Raw:      "",
			Fields:   schemaToBodyFields(schema, resolver),
		}
	}
	return interfaces.RequestBodyConfig{
		Format:   "raw",
		DataType: "JSON",
		Raw:      schemaExampleJSON(schema, resolver),
		Fields:   []interfaces.BodyFieldItem{},
	}
}

func schemaToBodyFields(schema map[string]any, resolver *openAPIResolver) []interfaces.BodyFieldItem {
	schema = resolver.resolveMap(schema)
	if schema == nil {
		return []interfaces.BodyFieldItem{}
	}
	requiredSet := map[string]struct{}{}
	if req, ok := schema["required"].([]any); ok {
		for _, item := range req {
			if s, ok := item.(string); ok {
				requiredSet[s] = struct{}{}
			}
		}
	}
	props, _ := schema["properties"].(map[string]any)
	fields := make([]interfaces.BodyFieldItem, 0, len(props))
	for name, propVal := range props {
		prop, _ := propVal.(map[string]any)
		prop = resolver.resolveMap(prop)
		_, required := requiredSet[name]
		enabled := true
		typ := openAPISchemaType(prop)
		example := openAPIExampleString(prop["example"])
		if example == "" {
			example = openAPIExampleString(prop["default"])
		}
		field := interfaces.BodyFieldItem{
			ParentID:    0,
			Name:        name,
			Type:        typ,
			Required:    required,
			Enabled:     &enabled,
			Description: stringValue(prop["description"]),
			Example:     example,
			Children:    []interfaces.BodyFieldItem{},
		}
		if typ == "object" {
			field.Children = schemaToBodyFields(prop, resolver)
		} else if typ == "array" {
			if items, ok := prop["items"].(map[string]any); ok {
				items = resolver.resolveMap(items)
				if openAPISchemaType(items) == "object" {
					field.Children = schemaToBodyFields(items, resolver)
				}
			}
		}
		fields = append(fields, field)
	}
	return fields
}

func schemaHasProperties(schema map[string]any) bool {
	if schema == nil {
		return false
	}
	props, _ := schema["properties"].(map[string]any)
	return len(props) > 0
}

func schemaExampleJSON(schema map[string]any, resolver *openAPIResolver) string {
	v := schemaToExampleValue(schema, resolver, 0)
	if v == nil {
		return ""
	}
	return prettyJSON(v)
}

func schemaToExampleValue(schema map[string]any, resolver *openAPIResolver, depth int) any {
	if schema == nil || depth > 6 {
		return nil
	}
	schema = resolver.resolveMap(schema)
	if ex, ok := schema["example"]; ok {
		return ex
	}
	if def, ok := schema["default"]; ok {
		return def
	}
	typ := openAPISchemaType(schema)
	switch typ {
	case "object":
		props, _ := schema["properties"].(map[string]any)
		obj := map[string]any{}
		for name, propVal := range props {
			prop, _ := propVal.(map[string]any)
			obj[name] = schemaToExampleValue(prop, resolver, depth+1)
		}
		if len(obj) == 0 {
			return map[string]any{}
		}
		return obj
	case "array":
		items, _ := schema["items"].(map[string]any)
		item := schemaToExampleValue(items, resolver, depth+1)
		if item == nil {
			return []any{}
		}
		return []any{item}
	case "integer", "number":
		return 0
	case "boolean":
		return false
	default:
		if enums, ok := schema["enum"].([]any); ok && len(enums) > 0 {
			return enums[0]
		}
		return ""
	}
}

func openAPISchemaType(schema map[string]any) string {
	if schema == nil {
		return "string"
	}
	if t, ok := schema["type"].(string); ok && t != "" {
		switch strings.ToLower(t) {
		case "integer", "number":
			return "number"
		case "boolean":
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
	if _, ok := schema["properties"]; ok {
		return "object"
	}
	if _, ok := schema["items"]; ok {
		return "array"
	}
	return "string"
}

func openAPIParamType(param map[string]any) string {
	if schema, ok := param["schema"].(map[string]any); ok {
		return openAPISchemaType(schema)
	}
	t := strings.ToLower(stringValue(param["type"]))
	switch t {
	case "integer", "number":
		return "number"
	case "boolean":
		return "boolean"
	case "array":
		return "array"
	case "file":
		return "file"
	default:
		return "string"
	}
}

func emptyRequestBody() interfaces.RequestBodyConfig {
	return interfaces.RequestBodyConfig{
		Format:   "json",
		DataType: "Object",
		Raw:      "",
		Fields:   []interfaces.BodyFieldItem{},
	}
}

func stringValue(v any) string {
	switch t := v.(type) {
	case string:
		return t
	case float64:
		return strconv.FormatFloat(t, 'f', -1, 64)
	case bool:
		return strconv.FormatBool(t)
	case nil:
		return ""
	default:
		return fmt.Sprint(t)
	}
}

func openAPIExampleString(v any) string {
	if v == nil {
		return ""
	}
	switch t := v.(type) {
	case string:
		return t
	case float64, bool:
		return stringValue(t)
	default:
		return prettyJSON(t)
	}
}

func prettyJSON(v any) string {
	switch t := v.(type) {
	case string:
		return t
	case json.Number:
		return t.String()
	case nil:
		return ""
	}
	b, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return fmt.Sprint(v)
	}
	return string(b)
}

type openAPIResolver struct {
	root map[string]any
}

func newOpenAPIResolver(root map[string]any) *openAPIResolver {
	if root == nil {
		root = map[string]any{}
	}
	return &openAPIResolver{root: root}
}

func (r *openAPIResolver) resolve(raw json.RawMessage) map[string]any {
	var obj map[string]any
	if err := json.Unmarshal(raw, &obj); err != nil {
		return map[string]any{}
	}
	return r.resolveMap(obj)
}

func (r *openAPIResolver) resolveMap(obj map[string]any) map[string]any {
	if obj == nil {
		return nil
	}
	ref, _ := obj["$ref"].(string)
	if ref == "" {
		return obj
	}
	target := r.lookupRef(ref)
	if target == nil {
		return obj
	}
	// Shallow merge: local keys override ref target.
	out := map[string]any{}
	for k, v := range target {
		out[k] = v
	}
	for k, v := range obj {
		if k == "$ref" {
			continue
		}
		out[k] = v
	}
	return out
}

func (r *openAPIResolver) lookupRef(ref string) map[string]any {
	ref = strings.TrimSpace(ref)
	if !strings.HasPrefix(ref, "#/") {
		return nil
	}
	parts := strings.Split(strings.TrimPrefix(ref, "#/"), "/")
	var cur any = r.root
	for _, part := range parts {
		part = strings.ReplaceAll(part, "~1", "/")
		part = strings.ReplaceAll(part, "~0", "~")
		m, ok := cur.(map[string]any)
		if !ok {
			return nil
		}
		cur, ok = m[part]
		if !ok {
			return nil
		}
	}
	out, _ := cur.(map[string]any)
	return out
}
