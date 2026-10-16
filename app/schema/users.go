package schema

import (
	"nest-api/app/schema/mixin"

	"nest-api/internal/utils"

	"entgo.io/ent"

	"entgo.io/ent/dialect"
	"entgo.io/ent/dialect/entsql"

	"entgo.io/ent/schema/edge"
	"entgo.io/ent/schema/field"
	"entgo.io/ent/schema/index"
)

type User struct {
	ent.Schema
}

func (User) Mixin() []ent.Mixin {
	return []ent.Mixin{
		mixin.SoftDeleteMixin{},
	}
}

func (User) Fields() []ent.Field {

	return []ent.Field{

		field.Int64("id").
			Positive().
			Unique().
			Comment("主键"),

		field.String("name").
			MaxLen(50).
			Default("").
			Comment("显示名称"),

		field.String("account").
			MaxLen(50).
			Comment("登录账号"),

		field.String("email").
			MaxLen(100).
			Default("").
			Comment("邮箱"),

		field.String("avatar").
			MaxLen(1024).
			Default("").
			Comment("头像地址"),

		field.String("mobile").
			MaxLen(20).
			Default("").
			Comment("手机号"),

		field.String("password").
			MaxLen(255).
			Default("").
			Sensitive().
			Comment("密码"),

		field.Bool("is_admin").
			Default(false).
			Comment("是否管理员"),

		field.Int8("status").
			Default(1).
			Comment("状态"),

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

func (User) Edges() []ent.Edge {
	return []ent.Edge{
		edge.To("owned_workspaces", Workspace.Type),
		edge.To("workspace_memberships", WorkspaceMember.Type),
		edge.To("created_projects", Project.Type),
		edge.To("created_folders", Folder.Type),
		edge.To("created_interfaces", API.Type),
		edge.To("updated_interfaces", API.Type),
		edge.To("created_environments", Environment.Type),
		edge.To("created_environment_variables", EnvironmentVariable.Type),
		edge.To("created_project_shares", ProjectShare.Type),
		edge.To("created_workspace_invites", WorkspaceInvite.Type),
		edge.To("project_memberships", ProjectMember.Type),
	}
}

func (User) Indexes() []ent.Index {
	return []ent.Index{
		index.Fields("account").
			Unique().
			Annotations(entsql.IndexWhere("deleted_at IS NULL")),
	}
}
