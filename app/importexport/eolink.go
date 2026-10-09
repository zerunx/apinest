package importexport

import (
	"encoding/json"
	"strconv"
	"strings"
	"time"

	"nest-api/app/interfaces"
	bizerr "nest-api/pkg/errors"
)

type eoGroup struct {
	GroupID        int64           `json:"groupID"`
	GroupName      string          `json:"groupName"`
	APIList        []eoAPI         `json:"apiList"`
	ChildGroupList []eoGroup       `json:"childGroupList"`
	ProjectInfo    *eoProjectInfo  `json:"projectInfo"`
}

type eoProjectInfo struct {
	ProjectName string `json:"projectName"`
}

type eoAPI struct {
	APIType      string          `json:"apiType"`
	BaseInfo     eoBaseInfo      `json:"baseInfo"`
	HeaderInfo   []eoHeader      `json:"headerInfo"`
	URLParam     []eoParam       `json:"urlParam"`
	RequestInfo  []eoParam       `json:"requestInfo"`
	RestfulParam []eoParam       `json:"restfulParam"`
	ResultInfo   []eoResult      `json:"resultInfo"`
}

type eoBaseInfo struct {
	APIName               string `json:"apiName"`
	APIURI                string `json:"apiURI"`
	APIRequestType        any    `json:"apiRequestType"`
	APIProtocol           any    `json:"apiProtocol"`
	APIRequestParamType   any    `json:"apiRequestParamType"`
	APIRequestRaw         string `json:"apiRequestRaw"`
	APISuccessMock        string `json:"apiSuccessMock"`
	APIFailureMock        string `json:"apiFailureMock"`
	APISuccessStatusCode  any    `json:"apiSuccessStatusCode"`
	APIFailureStatusCode  any    `json:"apiFailureStatusCode"`
	APISuccessContentType string `json:"apiSuccessContentType"`
	APIFailureContentType string `json:"apiFailureContentType"`
}

type eoHeader struct {
	HeaderName   string `json:"headerName"`
	HeaderValue  string `json:"headerValue"`
	ParamNotNull any    `json:"paramNotNull"`
	ParamName    string `json:"paramName"`
	ParamNote    string `json:"paramNote"`
}

type eoParam struct {
	ParamKey      string    `json:"paramKey"`
	ParamName     string    `json:"paramName"`
	ParamType     any       `json:"paramType"`
	ParamValue    string    `json:"paramValue"`
	ParamNotNull  any       `json:"paramNotNull"`
	ParamNote     string    `json:"paramNote"`
	ParamInfo     string    `json:"paramInfo"`
	ChildList     []eoParam `json:"childList"`
}

type eoResult struct {
	ResponseCode string    `json:"responseCode"`
	ResponseName string    `json:"responseName"`
	Raw          string    `json:"raw"`
	ParamList    []eoParam `json:"paramList"`
	IsDefault    any       `json:"isDefault"`
}

// ConvertEolinkDocument converts an Eolink group/project export JSON into ApiNest Document.
// Only groups and APIs are imported; scripts/cases/models are ignored.
func ConvertEolinkDocument(raw json.RawMessage) (*Document, error) {
	var root eoGroup
	if err := json.Unmarshal(raw, &root); err != nil {
		return nil, bizerr.New("import file is not valid JSON")
	}
	if !isEolinkDocument(root) {
		// Some exports wrap as array of groups.
		var groups []eoGroup
		if err := json.Unmarshal(raw, &groups); err == nil && len(groups) > 0 && isEolinkDocument(groups[0]) {
			children := make([]Node, 0, len(groups))
			for _, g := range groups {
				children = append(children, convertEoGroup(g))
			}
			return &Document{
				Format:     FormatApinest,
				Version:    Version,
				ExportedAt: time.Now().UTC().Format(time.RFC3339),
				Scope:      ScopeProject,
				Tree:       children,
			}, nil
		}
		return nil, bizerr.New("not an Eolink export file")
	}

	tree := []Node{convertEoGroup(root)}
	name := strings.TrimSpace(root.GroupName)
	if root.ProjectInfo != nil && strings.TrimSpace(root.ProjectInfo.ProjectName) != "" {
		name = strings.TrimSpace(root.ProjectInfo.ProjectName)
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

func isEolinkDocument(g eoGroup) bool {
	if strings.TrimSpace(g.GroupName) != "" {
		return true
	}
	if g.GroupID > 0 {
		return true
	}
	return len(g.APIList) > 0 || len(g.ChildGroupList) > 0
}

func convertEoGroup(g eoGroup) Node {
	children := make([]Node, 0, len(g.ChildGroupList)+len(g.APIList))
	for _, child := range g.ChildGroupList {
		children = append(children, convertEoGroup(child))
	}
	for _, api := range g.APIList {
		if node, ok := convertEoAPI(api); ok {
			children = append(children, node)
		}
	}
	return Node{
		Type:     "folder",
		Name:     fallbackName(g.GroupName, "未命名文件夹"),
		Children: children,
	}
}

func convertEoAPI(api eoAPI) (Node, bool) {
	apiType := strings.ToLower(strings.TrimSpace(api.APIType))
	name := strings.TrimSpace(api.BaseInfo.APIName)
	urlStr := strings.TrimSpace(api.BaseInfo.APIURI)
	method := eoMethod(api.BaseInfo.APIRequestType, apiType, api.BaseInfo.APIProtocol)

	// Skip empty removed-looking placeholders without name/url.
	if name == "" && urlStr == "" {
		return Node{}, false
	}

	headers := convertEoHeaders(api.HeaderInfo)
	query := convertEoParams(api.URLParam)
	body := convertEoBody(api.BaseInfo, api.RequestInfo)
	examples := convertEoResults(api.ResultInfo, api.BaseInfo)

	return Node{
		Type:             "api",
		Name:             fallbackName(name, method+" "+urlStr),
		Method:           method,
		URL:              urlStr,
		Status:           3,
		RequestHeaders:   headers,
		RequestBody:      body,
		QueryParams:      query,
		ResponseExamples: examples,
	}, true
}

func eoMethod(requestType any, apiType string, protocol any) string {
	if strings.Contains(apiType, "websocket") || eoInt(protocol) == 2 {
		return "WS"
	}
	switch eoInt(requestType) {
	case 0:
		return "POST"
	case 1:
		return "GET"
	case 2:
		return "PUT"
	case 3:
		return "DELETE"
	case 4:
		return "HEAD"
	case 5:
		return "OPTIONS"
	case 6:
		return "PATCH"
	default:
		if apiType != "" && apiType != "http" {
			return strings.ToUpper(apiType)
		}
		return "GET"
	}
}

func convertEoHeaders(items []eoHeader) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.HeaderName)
		if key == "" {
			continue
		}
		enabled := true
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        "string",
			Required:    eoTruthy(item.ParamNotNull),
			Enabled:     &enabled,
			Description: firstNonEmpty(item.ParamNote, item.ParamName),
			Example:     item.HeaderValue,
		})
	}
	return out
}

func convertEoParams(items []eoParam) []interfaces.ParamItem {
	out := make([]interfaces.ParamItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.ParamKey)
		if key == "" {
			continue
		}
		enabled := true
		example := item.ParamValue
		if example == "" {
			example = item.ParamInfo
		}
		out = append(out, interfaces.ParamItem{
			Name:        key,
			Type:        eoParamType(item.ParamType),
			Required:    eoTruthy(item.ParamNotNull),
			Enabled:     &enabled,
			Description: firstNonEmpty(item.ParamNote, item.ParamName),
			Example:     example,
		})
	}
	return out
}

func convertEoBody(base eoBaseInfo, requestInfo []eoParam) interfaces.RequestBodyConfig {
	paramType := eoInt(base.APIRequestParamType)
	raw := strings.TrimSpace(base.APIRequestRaw)

	switch paramType {
	case 0: // form-data
		return interfaces.RequestBodyConfig{
			Format:   "form-data",
			DataType: "Object",
			Raw:      "",
			Fields:   convertEoBodyFields(requestInfo),
		}
	case 2: // json
		if raw != "" {
			return interfaces.RequestBodyConfig{
				Format:   "raw",
				DataType: "JSON",
				Raw:      raw,
				Fields:   []interfaces.BodyFieldItem{},
			}
		}
		if len(requestInfo) > 0 {
			return interfaces.RequestBodyConfig{
				Format:   "json",
				DataType: "Object",
				Raw:      "",
				Fields:   convertEoBodyFields(requestInfo),
			}
		}
		return emptyRequestBody()
	case 3: // xml
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "XML",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	case 1: // raw
		return interfaces.RequestBodyConfig{
			Format:   "raw",
			DataType: "Text",
			Raw:      raw,
			Fields:   []interfaces.BodyFieldItem{},
		}
	default:
		if len(requestInfo) > 0 {
			// Heuristic: nested object fields => json schema; flat => form-data
			hasNested := false
			for _, item := range requestInfo {
				if len(item.ChildList) > 0 || eoParamType(item.ParamType) == "object" || eoParamType(item.ParamType) == "array" {
					hasNested = true
					break
				}
			}
			if hasNested {
				return interfaces.RequestBodyConfig{
					Format:   "json",
					DataType: "Object",
					Raw:      raw,
					Fields:   convertEoBodyFields(requestInfo),
				}
			}
			return interfaces.RequestBodyConfig{
				Format:   "form-data",
				DataType: "Object",
				Raw:      "",
				Fields:   convertEoBodyFields(requestInfo),
			}
		}
		if raw != "" {
			return interfaces.RequestBodyConfig{
				Format:   "raw",
				DataType: "JSON",
				Raw:      raw,
				Fields:   []interfaces.BodyFieldItem{},
			}
		}
		return emptyRequestBody()
	}
}

func convertEoBodyFields(items []eoParam) []interfaces.BodyFieldItem {
	out := make([]interfaces.BodyFieldItem, 0, len(items))
	for _, item := range items {
		key := strings.TrimSpace(item.ParamKey)
		if key == "" {
			continue
		}
		enabled := true
		example := item.ParamValue
		if example == "" {
			example = item.ParamInfo
		}
		field := interfaces.BodyFieldItem{
			ParentID:    0,
			Name:        key,
			Type:        eoParamType(item.ParamType),
			Required:    eoTruthy(item.ParamNotNull),
			Enabled:     &enabled,
			Description: firstNonEmpty(item.ParamNote, item.ParamName),
			Example:     example,
			Children:    convertEoBodyFields(item.ChildList),
		}
		out = append(out, field)
	}
	return out
}

func convertEoResults(results []eoResult, base eoBaseInfo) []interfaces.ResponseExampleItem {
	out := make([]interfaces.ResponseExampleItem, 0, len(results)+2)
	for _, item := range results {
		name := strings.TrimSpace(item.ResponseName)
		if name == "" {
			name = "响应"
		}
		code := 200
		if n, err := strconv.Atoi(strings.TrimSpace(item.ResponseCode)); err == nil {
			code = n
		}
		raw := strings.TrimSpace(item.Raw)
		if raw == "" && len(item.ParamList) > 0 {
			raw = prettyJSON(eoParamsToExample(item.ParamList))
		}
		if raw == "" {
			continue
		}
		out = append(out, interfaces.ResponseExampleItem{
			Name:        name,
			StatusCode:  code,
			ContentType: firstNonEmpty(base.APISuccessContentType, "application/json"),
			Raw:         raw,
		})
	}
	if len(out) == 0 {
		if s := strings.TrimSpace(base.APISuccessMock); s != "" {
			code := eoInt(base.APISuccessStatusCode)
			if code == 0 {
				code = 200
			}
			out = append(out, interfaces.ResponseExampleItem{
				Name:        "成功",
				StatusCode:  code,
				ContentType: firstNonEmpty(base.APISuccessContentType, "application/json"),
				Raw:         s,
			})
		}
		if s := strings.TrimSpace(base.APIFailureMock); s != "" {
			code := eoInt(base.APIFailureStatusCode)
			if code == 0 {
				code = 200
			}
			out = append(out, interfaces.ResponseExampleItem{
				Name:        "失败",
				StatusCode:  code,
				ContentType: firstNonEmpty(base.APIFailureContentType, "application/json"),
				Raw:         s,
			})
		}
	}
	return out
}

func eoParamsToExample(items []eoParam) any {
	obj := map[string]any{}
	for _, item := range items {
		key := strings.TrimSpace(item.ParamKey)
		if key == "" {
			continue
		}
		typ := eoParamType(item.ParamType)
		switch typ {
		case "object":
			obj[key] = eoParamsToExample(item.ChildList)
		case "array":
			if len(item.ChildList) > 0 {
				obj[key] = []any{eoParamsToExample(item.ChildList)}
			} else if item.ParamValue != "" {
				var arr any
				if err := json.Unmarshal([]byte(item.ParamValue), &arr); err == nil {
					obj[key] = arr
				} else {
					obj[key] = []any{}
				}
			} else {
				obj[key] = []any{}
			}
		case "number":
			if n, err := strconv.ParseFloat(item.ParamValue, 64); err == nil {
				obj[key] = n
			} else {
				obj[key] = 0
			}
		case "boolean":
			obj[key] = item.ParamValue == "true" || item.ParamValue == "1"
		default:
			if item.ParamValue != "" {
				obj[key] = item.ParamValue
			} else {
				obj[key] = ""
			}
		}
	}
	return obj
}

func eoParamType(v any) string {
	switch eoInt(v) {
	case 1:
		return "file"
	case 3, 4, 5, 9, 10, 11, 14:
		return "number"
	case 8:
		return "boolean"
	case 12:
		return "array"
	case 13:
		return "object"
	default:
		return "string"
	}
}

func eoInt(v any) int {
	switch t := v.(type) {
	case float64:
		return int(t)
	case int:
		return t
	case int64:
		return int(t)
	case json.Number:
		n, _ := t.Int64()
		return int(n)
	case string:
		n, _ := strconv.Atoi(strings.TrimSpace(t))
		return n
	default:
		return 0
	}
}

func eoTruthy(v any) bool {
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

func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if strings.TrimSpace(v) != "" {
			return strings.TrimSpace(v)
		}
	}
	return ""
}
