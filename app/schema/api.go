package schema

import (
	"nest-api/app/schema/mixin"
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"
	"entgo.io/ent/schema"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

type API struct {
	ent.Schema
}

func (API) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entsql.Annotation{Table: "interfaces"},
	}
}

func (API) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (API) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("project_id").
			Default(0).
			Comment("项目 ID"),

		field.Int64("folder_id").
			Default(0).
			Comment("所属文件夹 ID"),

		field.String("name").
			MaxLen(100).
			Default("").
			Comment("接口名称"),

		field.String("method").
			MaxLen(10).
			Default("").
			Comment("HTTP 方法"),

		field.String("protocol").
			MaxLen(10).
			Default("").
			Comment("请求协议：HTTP, HTTPS, WS, TCP, UDP, GRPC"),

		field.String("url").
			MaxLen(500).
			Default("").
			Comment("接口路径"),

		field.Uint8("status").
			Default(3).
			Comment("状态：1: 已发布, 2: 联调中, 3: 开发中, 6: 已废弃"),

		field.String("request_body_format").
			MaxLen(20).
			Default("json").
			Comment("请求体格式：form-data, json, xml, raw, binary"),

		field.String("request_body_data_type").
			MaxLen(20).
			Default("Object").
			Comment("请求体数据类型"),

		field.Text("request_body_raw").
			Default("").
			Comment("Raw 请求体内容"),

		field.Int("sort_order").
			Default(0).
			Comment("排序"),

		field.Int64("created_by").
			Default(0).
			Comment("创建者用户 ID"),

		field.Int64("updated_by").
			Default(0).
			Comment("最后修改者用户 ID"),

		field.Time("created_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("创建时间"),

		field.Time("updated_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			UpdateDefault(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("更新时间"),

		field.Time("deleted_at").
			Optional().
			Nillable().
			GoType(utils.DateTime{}).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("删除时间"),
	}
}

func (API) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("project", Project.Type).
			Ref("interfaces").
			Unique().
			Required().
			Field("project_id"),
		edge.From("folder", Folder.Type).
			Ref("interfaces").
			Unique().
			Required().
			Field("folder_id"),
		edge.From("creator", User.Type).
			Ref("created_interfaces").
			Unique().
			Required().
			Field("created_by"),
		edge.From("updater", User.Type).
			Ref("updated_interfaces").
			Unique().
			Required().
			Field("updated_by"),
		edge.To("response_headers", InterfaceHeader.Type),
		edge.To("response_results", InterfaceResult.Type),
		edge.To("response_examples", InterfaceExample.Type),
		edge.To("request_headers", InterfaceRequestHeader.Type),
		edge.To("query_params", InterfaceQueryParam.Type),
		edge.To("body_fields", InterfaceBodyField.Type),
		edge.To("share_items", ProjectShareInterface.Type),
	}
}
