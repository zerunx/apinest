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

type Environment struct {
	ent.Schema
}

func (Environment) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entsql.Annotation{Table: "environments"},
	}
}

func (Environment) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (Environment) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("project_id").
			Default(0).
			Comment("项目 ID"),

		field.String("name").
			MaxLen(100).
			Default("").
			Comment("环境名称"),

		field.String("remark").
			StorageKey("base_url").
			MaxLen(1024).
			Default("").
			Comment("备注"),

		field.Bool("is_default").
			Default(false).
			Comment("是否默认环境"),

		field.Int64("created_by").
			Default(0).
			Comment("创建者用户 ID"),

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

func (Environment) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("project", Project.Type).
			Ref("environments").
			Unique().
			Required().
			Field("project_id"),
		edge.From("creator", User.Type).
			Ref("created_environments").
			Unique().
			Required().
			Field("created_by"),
		edge.To("variables", EnvironmentVariable.Type),
	}
}
