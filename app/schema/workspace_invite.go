package schema

import (
	"nest-api/app/schema/mixin"
	"nest-api/internal/utils"

	"entgo.io/ent"
	"entgo.io/ent/dialect"
	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"entgo.io/ent/schema/index"
)

type WorkspaceInvite struct {
	ent.Schema
}

func (WorkspaceInvite) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (WorkspaceInvite) Fields() []ent.Field {
	return []ent.Field{
		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.Int64("workspace_id").
			Default(0).
			Comment("工作空间 ID"),

		field.String("invite_code").
			MaxLen(32).
			Unique().
			Comment("邀请码"),

		field.Uint8("role").
			Default(4).
			Comment("加入后角色：3=编辑者 4=查看者"),

		field.Int64("created_by").
			Default(0).
			Comment("创建者用户 ID"),

		field.Time("expires_at").
			GoType(utils.DateTime{}).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("过期时间"),

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

func (WorkspaceInvite) Edges() []ent.Edge {
	return []ent.Edge{
		edge.From("workspace", Workspace.Type).
			Ref("invites").
			Unique().
			Required().
			Field("workspace_id"),
		edge.From("creator", User.Type).
			Ref("created_workspace_invites").
			Unique().
			Required().
			Field("created_by"),
	}
}

func (WorkspaceInvite) Indexes() []ent.Index {
	return []ent.Index{
		index.Fields("workspace_id", "role"),
	}
}
