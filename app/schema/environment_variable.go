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

type EnvironmentVariable struct {
	ent.Schema
}

func (EnvironmentVariable) Annotations() []schema.Annotation {
	return []schema.Annotation{
		entsql.Annotation{Table: "environment_variables"},
	}
}

func (EnvironmentVariable) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (EnvironmentVariable) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("environment_id").
			Default(0).
			Comment("环境 ID"),

		field.String("key").
			MaxLen(200).
			Default("").
			Comment("变量名"),

		field.String("value").
			MaxLen(2000).
			Default("").
			Comment("变量值"),

		field.String("description").
			MaxLen(1024).
			Default("").
			Comment("描述"),

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

func (EnvironmentVariable) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("environment", Environment.Type).
			Ref("variables").
			Unique().
			Required().
			Field("environment_id"),
		edge.From("creator", User.Type).
			Ref("created_environment_variables").
			Unique().
			Required().
			Field("created_by"),
	}
}
