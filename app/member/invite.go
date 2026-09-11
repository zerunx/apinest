package member

import (
	"context"
	"crypto/rand"
	"fmt"
	"math/big"
	"strings"
	"time"

	appworkspace "nest-api/app/workspace"
	"nest-api/internal/database"
	"nest-api/internal/ent"
	"nest-api/internal/ent/workspace"
	"nest-api/internal/ent/workspaceinvite"
	"nest-api/internal/ent/workspacemember"
	"nest-api/internal/runtime"
	"nest-api/internal/utils"
	bizerr "nest-api/pkg/errors"
)

const inviteCodeCharset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
const inviteCodeLength = 12
const inviteTTL = 7 * 24 * time.Hour

func generateInviteCode() (string, error) {
	b := make([]byte, inviteCodeLength)
	max := big.NewInt(int64(len(inviteCodeCharset)))
	for i := range b {
		n, err := rand.Int(rand.Reader, max)
		if err != nil {
			return "", err
		}
		b[i] = inviteCodeCharset[n.Int64()]
	}
	return string(b), nil
}

func buildInviteURL(code string) string {
	base := strings.TrimRight(runtime.SiteURL(), "/")
	if base == "" {
		base = "http://localhost:5173"
	}
	return fmt.Sprintf("%s/invite?inviteCode=%s", base, code)
}

func inviteExpiresAt() utils.DateTime {
	return utils.DateTime{Time: time.Now().Add(inviteTTL)}
}

func toInviteLinkResponse(ws *ent.Workspace, inv *ent.WorkspaceInvite) *InviteLinkResponse {
	return &InviteLinkResponse{
		WorkspaceID:   ws.ID,
		WorkspaceName: ws.Name,
		InviteCode:    inv.InviteCode,
		InviteURL:     buildInviteURL(inv.InviteCode),
		SiteURL:       runtime.SiteURL(),
		Role:          inv.Role,
		ExpiresAt:     inv.ExpiresAt.Format(utils.DateTimeFormat),
	}
}

func createInvite(ctx context.Context, workspaceID, userID int64, role uint8) (*ent.WorkspaceInvite, error) {
	for range 8 {
		code, err := generateInviteCode()
		if err != nil {
			return nil, err
		}
		exists, err := database.DB.WorkspaceInvite.
			Query().
			Where(workspaceinvite.InviteCodeEQ(code)).
			Exist(ctx)
		if err != nil {
			return nil, err
		}
		if exists {
			continue
		}
		return database.DB.WorkspaceInvite.
			Create().
			SetWorkspaceID(workspaceID).
			SetInviteCode(code).
			SetRole(role).
			SetCreatedBy(userID).
			SetExpiresAt(inviteExpiresAt()).
			Save(ctx)
	}
	return nil, bizerr.New("failed to generate invite code, please retry")
}

func findActiveInvite(ctx context.Context, workspaceID int64, role uint8) (*ent.WorkspaceInvite, error) {
	now := utils.Now()
	return database.DB.WorkspaceInvite.
		Query().
		Where(
			workspaceinvite.WorkspaceIDEQ(workspaceID),
			workspaceinvite.RoleEQ(role),
			workspaceinvite.ExpiresAtGT(now),
		).
		Order(ent.Desc(workspaceinvite.FieldID)).
		First(ctx)
}

func (Service) GetInviteLink(ctx context.Context, userID int64, params InviteLinkRequest) (*InviteLinkResponse, error) {
	if _, err := appworkspace.Require(ctx, userID, params.WorkspaceID, appworkspace.ActionMemberInvite); err != nil {
		return nil, err
	}
	role := params.Role
	if role == 0 {
		role = appworkspace.RoleViewer
	}
	if !appworkspace.IsInviteLinkRole(role) {
		return nil, bizerr.New("invite role must be editor or viewer")
	}

	ws, err := database.DB.Workspace.
		Query().
		Where(workspace.IDEQ(params.WorkspaceID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("workspace not found")
		}
		return nil, err
	}

	inv, err := findActiveInvite(ctx, params.WorkspaceID, role)
	if err != nil {
		if !ent.IsNotFound(err) {
			return nil, err
		}
		inv, err = createInvite(ctx, params.WorkspaceID, userID, role)
		if err != nil {
			return nil, err
		}
	}
	return toInviteLinkResponse(ws, inv), nil
}

func (Service) RefreshInviteLink(ctx context.Context, userID int64, params InviteLinkRequest) (*InviteLinkResponse, error) {
	if _, err := appworkspace.Require(ctx, userID, params.WorkspaceID, appworkspace.ActionMemberInvite); err != nil {
		return nil, err
	}
	role := params.Role
	if role == 0 {
		role = appworkspace.RoleViewer
	}
	if !appworkspace.IsInviteLinkRole(role) {
		return nil, bizerr.New("invite role must be editor or viewer")
	}

	ws, err := database.DB.Workspace.
		Query().
		Where(workspace.IDEQ(params.WorkspaceID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, bizerr.New("workspace not found")
		}
		return nil, err
	}

	_, err = database.DB.WorkspaceInvite.
		Update().
		Where(
			workspaceinvite.WorkspaceIDEQ(params.WorkspaceID),
			workspaceinvite.RoleEQ(role),
			workspaceinvite.ExpiresAtGT(utils.Now()),
		).
		SetExpiresAt(utils.Now()).
		Save(ctx)
	if err != nil {
		return nil, err
	}

	inv, err := createInvite(ctx, params.WorkspaceID, userID, role)
	if err != nil {
		return nil, err
	}
	return toInviteLinkResponse(ws, inv), nil
}

func loadInviteByCode(ctx context.Context, code string) (*ent.WorkspaceInvite, *ent.Workspace, error) {
	code = strings.TrimSpace(code)
	if code == "" {
		return nil, nil, bizerr.New("invalid invite code")
	}

	inv, err := database.DB.WorkspaceInvite.
		Query().
		Where(workspaceinvite.InviteCodeEQ(code)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil, bizerr.New("invite link is invalid or expired")
		}
		return nil, nil, err
	}
	if !inv.ExpiresAt.After(time.Now()) {
		return nil, nil, bizerr.New("invite link is invalid or expired")
	}

	ws, err := database.DB.Workspace.
		Query().
		Where(workspace.IDEQ(inv.WorkspaceID)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil, bizerr.New("workspace not found")
		}
		return nil, nil, err
	}
	return inv, ws, nil
}

func (Service) PreviewInvite(ctx context.Context, params InvitePreviewRequest) (*InvitePreviewResponse, error) {
	inv, ws, err := loadInviteByCode(ctx, params.InviteCode)
	if err != nil {
		return nil, err
	}
	return &InvitePreviewResponse{
		WorkspaceID:   ws.ID,
		WorkspaceName: ws.Name,
		InviteCode:    inv.InviteCode,
		Role:          inv.Role,
		ExpiresAt:     inv.ExpiresAt.Format(utils.DateTimeFormat),
	}, nil
}

func (Service) AcceptInvite(ctx context.Context, userID int64, params AcceptInviteRequest) (*AcceptInviteResponse, error) {
	inv, ws, err := loadInviteByCode(ctx, params.InviteCode)
	if err != nil {
		return nil, err
	}

	exists, err := database.DB.WorkspaceMember.
		Query().
		Where(
			workspacemember.WorkspaceIDEQ(ws.ID),
			workspacemember.UserIDEQ(userID),
		).
		Exist(ctx)
	if err != nil {
		return nil, err
	}
	if exists {
		return &AcceptInviteResponse{
			WorkspaceID:   ws.ID,
			WorkspaceName: ws.Name,
			AlreadyMember: true,
			Role:          inv.Role,
		}, nil
	}

	role := inv.Role
	if !appworkspace.IsInviteLinkRole(role) {
		role = appworkspace.RoleViewer
	}

	_, err = database.DB.WorkspaceMember.
		Create().
		SetWorkspaceID(ws.ID).
		SetUserID(userID).
		SetRole(role).
		Save(ctx)
	if err != nil {
		return nil, err
	}

	return &AcceptInviteResponse{
		WorkspaceID:   ws.ID,
		WorkspaceName: ws.Name,
		AlreadyMember: false,
		Role:          role,
	}, nil
}
