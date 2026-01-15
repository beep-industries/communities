#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::{
        friend::entities::UserId,
        member_role::ports::MemberRoleService,
        role::{
            entities::{CreateRoleRepoInput, Permissions},
            ports::RoleRepository,
        },
        server::{
            entities::{InsertServerInput, ServerVisibility},
            ports::ServerRepository,
        },
        server_member::{CreateMemberInput, MemberRepository},
        test::create_mock_service,
    };

    #[tokio::test]
    async fn test_assign_member_role_success() {
        let service = create_mock_service();
        let input = InsertServerInput {
            name: "Test Server".to_string(),
            owner_id: UserId::from(Uuid::new_v4()),
            picture_url: Some("https://example.com/picture.png".to_string()),
            banner_url: Some("https://example.com/banner.png".to_string()),
            description: Some("A test server".to_string()),
            visibility: ServerVisibility::Public,
        };

        let server = service
            .server_repository
            .insert(input)
            .await
            .expect("create_server returned an error");

        let create_role_input = CreateRoleRepoInput {
            server_id: *server.id,
            name: "test".to_string(),
            permissions: Permissions(0x1),
        };

        let role = service
            .role_repository
            .create(create_role_input)
            .await
            .expect("Could not create role");
        let input = CreateMemberInput {
            server_id: server.id,
            user_id: UserId(Uuid::new_v4()),
            nickname: None,
        };

        let server_member = service
            .member_repository
            .insert(input)
            .await
            .expect("Could not create server member");
        let member_role = service
            .assign_member_to_role(role.id, server_member.id)
            .await
            .expect("Could not create member");

        assert_eq!(member_role.member_id, server_member.id);
        assert_eq!(member_role.role_id, role.id);
    }

    #[tokio::test]
    async fn test_unassign_member_role_success() {
        let service = create_mock_service();
        let input = InsertServerInput {
            name: "Test Server".to_string(),
            owner_id: UserId::from(Uuid::new_v4()),
            picture_url: Some("https://example.com/picture.png".to_string()),
            banner_url: Some("https://example.com/banner.png".to_string()),
            description: Some("A test server".to_string()),
            visibility: ServerVisibility::Public,
        };

        let server = service
            .server_repository
            .insert(input)
            .await
            .expect("create_server returned an error");

        let create_role_input = CreateRoleRepoInput {
            server_id: *server.id,
            name: "test".to_string(),
            permissions: Permissions(0x1),
        };

        let role = service
            .role_repository
            .create(create_role_input)
            .await
            .expect("Could not create role");
        let input = CreateMemberInput {
            server_id: server.id,
            user_id: UserId(Uuid::new_v4()),
            nickname: None,
        };

        let server_member = service
            .member_repository
            .insert(input)
            .await
            .expect("Could not create server member");

        let member_role = service
            .assign_member_to_role(role.id, server_member.id)
            .await
            .expect("Could not create member");
        service
            .unassign_member_from_role(member_role.role_id, member_role.member_id)
            .await
            .expect("Member role should be deleted");
    }
}
