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
			Comment("ä¸»é®"),

		field.Int64("project_id").
			Default(0).
			Comment("é¡¹ç® ID"),

		field.Int64("folder_id").
			Default(0).
			Comment("æå±æä»¶å¤¹ ID"),

		field.String("name").
			MaxLen(100).
			Default("").
			Comment("æ¥å£åç§°"),

		field.String("method").
			MaxLen(10).
			Default("").
			Comment("HTTP æ¹æ³"),

		field.String("protocol").
			MaxLen(10).
			Default("").
			Comment("è¯·æ±åè®®ï¼HTTP, HTTPS, WS, TCP, UDP, GRPC"),

		field.String("url").
			MaxLen(1024).
			Default("").
			Comment("æ¥å£è·¯å¾"),

		field.Uint8("status").
			Default(3).
			Comment("ç¶æï¼1: å·²åå¸, 2: èè°ä¸­, 3: å¼åä¸­, 6: å·²åºå¼"),

		field.String("request_body_format").
			MaxLen(20).
			Default("json").
			Comment("è¯·æ±ä½æ ¼å¼ï¼form-data, json, xml, raw, binary"),

		field.String("request_body_data_type").
			MaxLen(20).
			Default("Object").
			Comment("è¯·æ±ä½æ°æ®ç±»å"),

		field.Text("request_body_raw").
			Default("").
			Comment("Raw è¯·æ±ä½åå®¹"),

		field.Int("sort_order").
			Default(0).
			Comment("æåº"),

		field.Int64("created_by").
			Default(0).
			Comment("åå»ºèç¨æ· ID"),

		field.Int64("updated_by").
			Default(0).
			Comment("æåä¿®æ¹èç¨æ· ID"),

		field.Time("created_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("åå»ºæ¶é´"),

		field.Time("updated_at").
			GoType(utils.DateTime{}).
			Default(utils.Now).
			UpdateDefault(utils.Now).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("æ´æ°æ¶é´"),

		field.Time("deleted_at").
			Optional().
			Nillable().
			GoType(utils.DateTime{}).
			SchemaType(map[string]string{
				dialect.Postgres: "timestamp(0) without time zone",
			}).
			Comment("å é¤æ¶é´"),
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
