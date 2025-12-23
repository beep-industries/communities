use chrono::{DateTime, Utc};
use permission_translation::models::CapilityHexValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::common::CoreError;

pub type RoleId = Uuid;

//
#[derive(Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct Permissions(pub i32);

#[derive(Clone)]
pub struct Role {
    pub id: RoleId,
    pub server_id: Uuid,
    pub name: String,
    pub permissions: Permissions,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct CreateRoleInput {
    pub server_id: Uuid,
    pub name: String,
    pub permissions: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateRoleRepoInput {
    pub server_id: Uuid,
    pub name: String,
    pub permissions: Permissions,
}

#[derive(Debug, Error)]
pub enum RoleError {
    #[error("Bad role payload: {msg}")]
    BadRolePayload { msg: String },
}

impl Into<CoreError> for RoleError {
    fn into(self) -> CoreError {
        match self {
            RoleError::BadRolePayload { msg } => CoreError::Error { msg },
        }
    }
}

impl TryFrom<CreateRoleInput> for CreateRoleRepoInput {
    type Error = PermissionError;

    fn try_from(value: CreateRoleInput) -> Result<Self, Self::Error> {
        Ok(Self {
            server_id: value.server_id,
            name: value.name,
            permissions: Permissions::try_from(value.permissions)?,
        })
    }
}

pub struct UpdateRoleInput {
    pub id: RoleId,
    pub name: Option<String>,
    pub permissions: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateRoleRepoInput {
    pub id: RoleId,
    pub name: Option<String>,
    pub permissions: Option<Permissions>,
}

impl TryFrom<UpdateRoleInput> for UpdateRoleRepoInput {
    type Error = PermissionError;

    fn try_from(value: UpdateRoleInput) -> Result<Self, Self::Error> {
        let permissions = match value.permissions {
            Some(perm) => Some(Permissions::try_from(perm)?),
            None => None,
        };

        Ok(Self {
            id: value.id,
            name: value.name,
            permissions,
        })
    }
}

#[derive(Debug)]
pub enum Permission {
    Administrator,    // Can do any action on any subject (channel, webhooks…) in a server.
    ManageServer,     // Can update a server (all CRUD except delete).
    ManageRoles,      // Can do all CRUD operations on all roles.
    CreateInvitation, // Can create server invites.
    ManageChannels,   // Can do all CRUD operations on every channel.
    ManageWebhooks,   // Can do all CRUD operations on every webhook.
    ViewChannels,     // Can see the channel and its contents (messages).
    SendMessages,     // Can send a message on the channel.
    ManageNicknames,  // Can update other users' nicknames.
    ChangeNickname,   // Can update your own nickname.
    ManageMessages,   // Can delete other users' messages.
    AttachFiles,      // Can upload images and files.
}

impl Into<CapilityHexValue> for Permission {
    fn into(self) -> CapilityHexValue {
        match self {
            Permission::Administrator => 0x1,
            Permission::ManageServer => 0x2,
            Permission::ManageRoles => 0x4,
            Permission::CreateInvitation => 0x8,
            Permission::ManageChannels => 0x10,
            Permission::ManageWebhooks => 0x20,
            Permission::ViewChannels => 0x40,
            Permission::SendMessages => 0x80,
            Permission::ManageNicknames => 0x100,
            Permission::ChangeNickname => 0x200,
            Permission::ManageMessages => 0x400,
            Permission::AttachFiles => 0x800,
        }
    }
}

impl TryFrom<i32> for Permission {
    type Error = PermissionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(Permission::Administrator),
            0x2 => Ok(Permission::ManageServer),
            0x4 => Ok(Permission::ManageRoles),
            0x8 => Ok(Permission::CreateInvitation),
            0x10 => Ok(Permission::ManageChannels),
            0x20 => Ok(Permission::ManageWebhooks),
            0x40 => Ok(Permission::ViewChannels),
            0x80 => Ok(Permission::SendMessages),
            0x100 => Ok(Permission::ManageNicknames),
            0x200 => Ok(Permission::ChangeNickname),
            0x400 => Ok(Permission::ManageMessages),
            0x800 => Ok(Permission::AttachFiles),
            _ => Err(PermissionError::BadFormat),
        }
    }
}

impl TryFrom<i32> for Permissions {
    type Error = PermissionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        // All valid permission bits combined
        const ALL_PERMISSIONS: i32 =
            0x1 | 0x2 | 0x4 | 0x8 | 0x10 | 0x20 | 0x40 | 0x80 | 0x100 | 0x200 | 0x400 | 0x800;

        // Check if value contains only valid permission bits
        if value & !ALL_PERMISSIONS != 0 {
            return Err(PermissionError::BadFormat);
        }

        Ok(Permissions(value))
    }
}

#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("The permissions you provided are not conform")]
    BadFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_try_from_single_valid_bits() {
        // Test all valid single permission bits
        assert!(matches!(
            Permission::try_from(0x1),
            Ok(Permission::Administrator)
        ));
        assert!(matches!(
            Permission::try_from(0x2),
            Ok(Permission::ManageServer)
        ));
        assert!(matches!(
            Permission::try_from(0x4),
            Ok(Permission::ManageRoles)
        ));
        assert!(matches!(
            Permission::try_from(0x8),
            Ok(Permission::CreateInvitation)
        ));
        assert!(matches!(
            Permission::try_from(0x10),
            Ok(Permission::ManageChannels)
        ));
        assert!(matches!(
            Permission::try_from(0x20),
            Ok(Permission::ManageWebhooks)
        ));
        assert!(matches!(
            Permission::try_from(0x40),
            Ok(Permission::ViewChannels)
        ));
        assert!(matches!(
            Permission::try_from(0x80),
            Ok(Permission::SendMessages)
        ));
        assert!(matches!(
            Permission::try_from(0x100),
            Ok(Permission::ManageNicknames)
        ));
        assert!(matches!(
            Permission::try_from(0x200),
            Ok(Permission::ChangeNickname)
        ));
        assert!(matches!(
            Permission::try_from(0x400),
            Ok(Permission::ManageMessages)
        ));
        assert!(matches!(
            Permission::try_from(0x800),
            Ok(Permission::AttachFiles)
        ));
    }

    #[test]
    fn test_permission_try_from_invalid_bits() {
        // Combined permissions should fail for single Permission
        assert!(Permission::try_from(0x3).is_err()); // Administrator | ManageServer
        assert!(Permission::try_from(0x5).is_err()); // Administrator | ManageRoles

        // Invalid bits should fail
        assert!(Permission::try_from(0x1000).is_err());
        assert!(Permission::try_from(0xFFFF).is_err());
        assert!(Permission::try_from(-1).is_err());
    }

    #[test]
    fn test_permissions_try_from_single_valid_bits() {
        // Single permission bits should work
        assert!(Permissions::try_from(0x1).is_ok());
        assert!(Permissions::try_from(0x2).is_ok());
        assert!(Permissions::try_from(0x800).is_ok());
    }

    #[test]
    fn test_permissions_try_from_combined_valid_bits() {
        // Combined permissions should work
        assert!(Permissions::try_from(0x3).is_ok()); // Administrator | ManageServer
        assert!(Permissions::try_from(0x803).is_ok()); // AttachFiles | ManageServer | Administrator
        assert!(Permissions::try_from(0xFF).is_ok()); // First 8 permissions
        assert!(Permissions::try_from(0xFFF).is_ok()); // All permissions
    }

    #[test]
    fn test_permissions_try_from_zero() {
        // Zero (no permissions) should be valid
        assert!(Permissions::try_from(0x0).is_ok());
    }

    #[test]
    fn test_permissions_try_from_invalid_bits() {
        // Invalid bits should fail
        assert!(Permissions::try_from(0x1000).is_err()); // Invalid bit
        assert!(Permissions::try_from(0x2000).is_err()); // Invalid bit
        assert!(Permissions::try_from(0x10000).is_err()); // Invalid bit
        assert!(Permissions::try_from(0x1003).is_err()); // Valid bits + invalid bit
        assert!(Permissions::try_from(-1).is_err()); // Negative value with invalid bits
    }

    #[test]
    fn test_permissions_try_from_all_permissions() {
        // All valid permissions combined should work
        let all_perms =
            0x1 | 0x2 | 0x4 | 0x8 | 0x10 | 0x20 | 0x40 | 0x80 | 0x100 | 0x200 | 0x400 | 0x800;
        assert!(Permissions::try_from(all_perms).is_ok());
        assert_eq!(all_perms, 0xFFF);
    }

    #[test]
    fn test_permission_into_capability_hex() {
        // Test conversion to hex values
        let admin: CapilityHexValue = Permission::Administrator.into();
        assert_eq!(admin, 0x1);

        let manage_server: CapilityHexValue = Permission::ManageServer.into();
        assert_eq!(manage_server, 0x2);

        let attach_files: CapilityHexValue = Permission::AttachFiles.into();
        assert_eq!(attach_files, 0x800);
    }
}
