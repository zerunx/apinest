package schema

import (
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"entgo.io/ent/schema/index"
)

type ProjectMember struct {
	ent.Schema
}

func (ProjectMember) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("workspace_id").
			Default(0).
			Comment("工作空间 ID"),

		field.Int64("project_id").
			Default(0).
			Comment("项目 ID"),

		field.Int64("user_id").
			Default(0).
			Comment("用户 ID"),

		field.Time("created_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("加入时间"),

		field.Time("updated_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			UpdateDefault(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("更新时间"),
	}
}

func (ProjectMember) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("workspace", Workspace.Type).
			Ref("project_members").
			Unique().
			Required().
			Field("workspace_id"),
		edge.From("project", Project.Type).
			Ref("members").
			Unique().
			Required().
			Field("project_id"),
		edge.From("user", User.Type).
			Ref("project_memberships").
			Unique().
			Required().
			Field("user_id"),
	}
}

func (ProjectMember) Indexes() []ent.Index {
	return []ent.Index{
		index.Fields("project_id", "user_id").Unique(),
		index.Fields("workspace_id", "user_id"),
	}
}
