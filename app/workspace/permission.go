package workspace

type Action string

const (
	ActionWorkspaceRead     Action = "workspace:read"
	ActionWorkspaceUpdate   Action = "workspace:update"
	ActionWorkspaceDelete   Action = "workspace:delete"
	ActionWorkspaceTransfer Action = "workspace:transfer"

	ActionMemberRead       Action = "member:read"
	ActionMemberInvite     Action = "member:invite"
	ActionMemberUpdateRole Action = "member:update_role"
	ActionMemberRemove     Action = "member:remove"

	ActionProjectRead   Action = "project:read"
	ActionProjectCreate Action = "project:create"
	ActionProjectUpdate Action = "project:update"
	ActionProjectDelete Action = "project:delete"
)

var rolePermissions = map[uint8]map[Action]struct{}{
	RoleOwner: permSet(
		ActionWorkspaceRead, ActionWorkspaceUpdate, ActionWorkspaceDelete, ActionWorkspaceTransfer,
		ActionMemberRead, ActionMemberInvite, ActionMemberUpdateRole, ActionMemberRemove,
		ActionProjectRead, ActionProjectCreate, ActionProjectUpdate, ActionProjectDelete,
	),
	RoleAdmin: permSet(
		ActionWorkspaceRead,
		ActionMemberRead, ActionMemberInvite, ActionMemberUpdateRole,
		ActionProjectRead, ActionProjectCreate, ActionProjectUpdate, ActionProjectDelete,
	),
	RoleEditor: permSet(
		ActionWorkspaceRead,
		ActionMemberRead,
		ActionProjectRead, ActionProjectCreate, ActionProjectUpdate, ActionProjectDelete,
	),
	RoleViewer: permSet(
		ActionWorkspaceRead,
		ActionMemberRead,
		ActionProjectRead,
	),
}

func permSet(actions ...Action) map[Action]struct{} {
	m := make(map[Action]struct{}, len(actions))
	for _, a := range actions {
		m[a] = struct{}{}
	}
	return m
}

func (a Action) Allowed(role uint8) bool {
	perms, ok := rolePermissions[role]
	if !ok {
		return false
	}
	_, ok = perms[a]
	return ok
}
