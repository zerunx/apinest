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

type InterfaceQueryParam struct {
	ent.Schema
}

func (InterfaceQueryParam) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entsql.Annotation{Table: "interface_query_params"},
	}
}

func (InterfaceQueryParam) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (InterfaceQueryParam) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").Positive().Unique().Comment("主键"),
		field.Int64("interface_id").Default(0).Comment("接口 ID"),
		field.String("name").MaxLen(100).Default("").Comment("参数名"),
		field.String("type").MaxLen(50).Default("string").Comment("类型"),
		field.Bool("required").Default(false).Comment("是否必填"),
		field.Bool("enabled").Default(true).Comment("是否启用"),
		field.String("description").MaxLen(1024).Default("").Comment("说明"),
		field.String("example").MaxLen(1024).Default("").Comment("参数示例"),
		field.Int("sort_order").Default(0).Comment("排序"),
		field.Time("created_at").GoType(utils.DateTime{}).Default(utils.Now).
			SchemaType(map[string]string{dialect.Postgres: "timestamp(0) without time zone"}).Comment("创建时间"),
		field.Time("updated_at").GoType(utils.DateTime{}).Default(utils.Now).UpdateDefault(utils.Now).
			SchemaType(map[string]string{dialect.Postgres: "timestamp(0) without time zone"}).Comment("更新时间"),
		field.Time("deleted_at").Optional().Nillable().GoType(utils.DateTime{}).
			SchemaType(map[string]string{dialect.Postgres: "timestamp(0) without time zone"}).Comment("删除时间"),
	}
}

func (InterfaceQueryParam) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("interface", API.Type).Ref("query_params").Unique().Required().Field("interface_id"),
	}
}
