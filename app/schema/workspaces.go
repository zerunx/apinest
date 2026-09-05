package schema

import (
	"nest-api/app/schema/mixin"
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
)

type Workspace struct {
	ent.Schema
}

func (Workspace) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (Workspace) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.String("name").
			MaxLen(100).
			Default("").
			Comment("工作空间名称"),

		field.Int64("owner_id").
			Default(0).
			Comment("拥有者用户 ID"),

		field.String("invite_code").
			MaxLen(12).
			Optional().
			Unique().
			Comment("成员邀请码"),

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

func (Workspace) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("owner", User.Type).
			Ref("owned_workspaces").
			Unique().
			Required().
			Field("owner_id"),
		edge.To("members", WorkspaceMember.Type),
		edge.To("projects", Project.Type),
		edge.To("invites", WorkspaceInvite.Type),
		edge.To("project_members", ProjectMember.Type),
	}
}
