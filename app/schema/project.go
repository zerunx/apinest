package schema

import (
	"nest-api/app/schema/mixin"
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

type Project struct {
	ent.Schema
}

func (Project) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (Project) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("workspace_id").
			Default(0).
			Comment("工作空间 ID"),

		field.String("name").
			MaxLen(100).
			Default("").
			Comment("项目名称"),

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

func (Project) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("workspace", Workspace.Type).
			Ref("projects").
			Unique().
			Required().
			Field("workspace_id"),
		edge.From("creator", User.Type).
			Ref("created_projects").
			Unique().
			Required().
			Field("created_by"),
		edge.To("folders", Folder.Type),
		edge.To("interfaces", API.Type),
		edge.To("environments", Environment.Type),
		edge.To("shares", ProjectShare.Type),
		edge.To("members", ProjectMember.Type),
	}
}
