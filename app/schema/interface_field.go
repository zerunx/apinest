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

type InterfaceField struct {
	ent.Schema
}

func (InterfaceField) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entsql.Annotation{Table: "interface_fields"},
	}
}

func (InterfaceField) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (InterfaceField) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").Positive().Unique().Comment("主键"),
		field.Int64("result_id").Default(0).Comment("返回结果 ID"),
		field.Int64("parent_id").Default(0).Comment("父字段 ID"),
		field.String("name").MaxLen(100).Default("").Comment("参数名"),
		field.String("type").MaxLen(50).Default("string").Comment("类型"),
		field.Bool("required").Default(false).Comment("是否必填"),
		field.String("description").MaxLen(1024).Default("").Comment("说明"),
		field.String("mock").MaxLen(1024).Default("").Comment("Mock"),
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

func (InterfaceField) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("result", InterfaceResult.Type).Ref("fields").Unique().Required().Field("result_id"),
	}
}
