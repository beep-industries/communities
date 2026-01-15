// == Create Role Tests ==

#[cfg(test)]
mod tests {
    use crate::domain::{
        common::GetPaginated,
        role::{
            entities::{CreateRoleInput, Permissions, RoleId, UpdateRoleInput},
            ports::{RoleRepository, RoleService},
        },
        test::create_mock_service,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_role_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Admin".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(), // Administrator permission
        };

        let role = service
            .create_role(input)
            .await
            .expect("create_role returned an error");

        assert_eq!(role.name, "Admin", "Expected correct role name");
        assert_eq!(role.server_id, server_id, "Expected correct server ID");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_role_with_multiple_permissions() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Moderator".to_string(),
            permissions: Permissions::try_from(0x4 | 0x10 | 0x400).unwrap(), // ManageRoles | ManageChannels | ManageMessages
        };

        let role = service
            .create_role(input)
            .await
            .expect("create_role returned an error");

        assert_eq!(role.name, "Moderator", "Expected correct role name");
        assert_eq!(role.server_id, server_id, "Expected correct server ID");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_role_with_zero_permissions() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Guest".to_string(),
            permissions: Permissions::try_from(0x0).unwrap(), // No permissions
        };

        let role = service
            .create_role(input)
            .await
            .expect("create_role returned an error");

        assert_eq!(role.name, "Guest", "Expected correct role name");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_role_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role using repository
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Test Role".to_string(),
            permissions: Permissions::try_from(0x2).unwrap(), // ManageServer
        };
        let created_role = service.create_role(input).await?;

        // Get the role
        let role = service
            .get_role(&created_role.id)
            .await
            .expect("get_role returned an error");

        assert_eq!(role.id, created_role.id, "Expected same role ID");
        assert_eq!(role.name, "Test Role", "Expected correct role name");
        assert_eq!(role.server_id, server_id, "Expected correct server ID");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_role_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let non_existent_id = RoleId::from(Uuid::new_v4());
        let result = service.get_role(&non_existent_id).await;

        assert!(result.is_err(), "get_role should have returned an error");

        match result {
            Err(error) => {
                assert!(
                    error.to_string().contains("not found"),
                    "Expected role not found error"
                );
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }

        Ok(())
    }

    // == List Roles by Server Tests ==

    #[tokio::test]
    async fn test_list_roles_by_server_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();

        // Insert multiple roles for the same server
        for i in 1..=3 {
            let input = CreateRoleInput {
                server_id,
                name: format!("Test Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            service.create_role(input).await?;
        }

        let (roles, total) = service
            .list_roles_by_server(&GetPaginated::default(), server_id)
            .await
            .expect("list_roles_by_server returned an error");

        assert_eq!(roles.len(), 3, "Expected 3 roles in the list");
        assert_eq!(total, 3, "Expected total count to be 3");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_roles_by_server_multiple_servers() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();

        let server_id_1 = Uuid::new_v4();
        let server_id_2 = Uuid::new_v4();

        // Insert roles for server 1
        for i in 1..=3 {
            let input = CreateRoleInput {
                server_id: server_id_1,
                name: format!("Server1 Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            service.create_role(input).await?;
        }

        // Insert roles for server 2
        for i in 1..=2 {
            let input = CreateRoleInput {
                server_id: server_id_2,
                name: format!("Server2 Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            service.create_role(input).await?;
        }

        // List roles for server 1
        let (roles1, total1) = service
            .list_roles_by_server(&GetPaginated::default(), server_id_1)
            .await
            .expect("list_roles_by_server returned an error");

        assert_eq!(roles1.len(), 3, "Expected 3 roles for server 1 in the list");
        assert_eq!(total1, 3, "Expected total count for server 1 to be 3");

        // List roles for server 2
        let (roles2, total2) = service
            .list_roles_by_server(&GetPaginated::default(), server_id_2)
            .await
            .expect("list_roles_by_server returned an error");

        assert_eq!(roles2.len(), 2, "Expected 2 roles for server 2 in the list");
        assert_eq!(total2, 2, "Expected total count for server 2 to be 2");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_roles_by_server_with_pagination() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();

        // Insert 25 roles
        for i in 1..=25 {
            let input = CreateRoleInput {
                server_id,
                name: format!("Test Role {}", i),
                permissions: Permissions::try_from(0x1).unwrap(),
            };
            service.create_role(input).await?;
        }

        // Test page 1
        let pagination1 = GetPaginated { page: 1, limit: 10 };
        let (roles1, total1) = service
            .list_roles_by_server(&pagination1, server_id)
            .await
            .expect("list_roles_by_server page 1 returned an error");

        assert_eq!(roles1.len(), 10, "Expected 10 roles on page 1");
        assert_eq!(total1, 25, "Expected total count to be 25");

        // Test page 2
        let pagination2 = GetPaginated { page: 2, limit: 10 };
        let (roles2, total2) = service
            .list_roles_by_server(&pagination2, server_id)
            .await
            .expect("list_roles_by_server page 2 returned an error");

        assert_eq!(roles2.len(), 10, "Expected 10 roles on page 2");
        assert_eq!(total2, 25, "Expected total count to be 25");

        // Test page 3
        let pagination3 = GetPaginated { page: 3, limit: 10 };
        let (roles3, total3) = service
            .list_roles_by_server(&pagination3, server_id)
            .await
            .expect("list_roles_by_server page 3 returned an error");

        assert_eq!(roles3.len(), 5, "Expected 5 roles on page 3");
        assert_eq!(total3, 25, "Expected total count to be 25");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_roles_by_server_empty() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();

        let (roles, total) = service
            .list_roles_by_server(&GetPaginated::default(), server_id)
            .await
            .expect("list_roles_by_server returned an error");

        assert_eq!(roles.len(), 0, "Expected empty role list");
        assert_eq!(total, 0, "Expected total count to be 0");

        Ok(())
    }

    // == Update Role Tests ==

    #[tokio::test]
    async fn test_update_role_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Original Role".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let created_role = service.create_role(input).await?;

        // Update the role
        let update_input = UpdateRoleInput {
            id: created_role.id.clone(),
            name: Some("Updated Role".to_string()),
            permissions: Some(0x2), // ManageServer
        };

        let updated_role = service
            .update_role(update_input)
            .await
            .expect("update_role returned an error");

        assert_eq!(updated_role.name, "Updated Role", "Expected updated name");
        assert!(
            updated_role.updated_at.is_some(),
            "Expected updated_at to be set"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_role_partial_update_name_only() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Original Role".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let created_role = service.create_role(input).await?;

        // Update only the name
        let update_input = UpdateRoleInput {
            id: created_role.id.clone(),
            name: Some("Updated Name Only".to_string()),
            permissions: None,
        };

        let updated_role = service
            .update_role(update_input)
            .await
            .expect("update_role returned an error");

        assert_eq!(
            updated_role.name, "Updated Name Only",
            "Expected updated name"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_role_partial_update_permissions_only()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Original Role".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let created_role = service.create_role(input).await?;

        // Update only the permissions
        let update_input = UpdateRoleInput {
            id: created_role.id.clone(),
            name: None,
            permissions: Some(0x4 | 0x10), // ManageRoles | ManageChannels
        };

        let updated_role = service
            .update_role(update_input)
            .await
            .expect("update_role returned an error");

        assert_eq!(
            updated_role.name, "Original Role",
            "Expected unchanged name"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_role_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let update_input = UpdateRoleInput {
            id: RoleId::from(Uuid::new_v4()),
            name: Some("Updated Role".to_string()),
            permissions: None,
        };

        let result = service.update_role(update_input).await;

        assert!(result.is_err(), "update_role should have returned an error");

        match result {
            Err(error) => {
                assert!(
                    error.to_string().contains("not found"),
                    "Expected role not found error"
                );
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_role_fail_invalid_permissions() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Original Role".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let created_role = service.create_role(input).await?;

        // Try to update with invalid permissions
        let update_input = UpdateRoleInput {
            id: created_role.id.clone(),
            name: None,
            permissions: Some(0x1000), // Invalid permission bit
        };

        let result = service.update_role(update_input).await;

        assert!(result.is_err(), "update_role should have returned an error");

        match result {
            Err(error) => {
                assert!(
                    error
                        .to_string()
                        .contains("permissions you provided are not conform"),
                    "Expected permissions error"
                );
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }

        Ok(())
    }

    // == Delete Role Tests ==

    #[tokio::test]
    async fn test_delete_role_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Insert a role
        let server_id = Uuid::new_v4();
        let input = CreateRoleInput {
            server_id,
            name: "Test Role".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let created_role = service.create_role(input).await?;

        // Delete the role
        service
            .delete_role(&created_role.id)
            .await
            .expect("delete_role returned an error");

        // Verify role is deleted - should return error
        let result = service.role_repository.find_by_id(&created_role.id).await;
        assert!(result.is_err(), "Expected role to be deleted");

        match result {
            Err(error) => {
                assert!(
                    error.to_string().contains("not found"),
                    "Expected not found error"
                );
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_role_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        let non_existent_id = RoleId::from(Uuid::new_v4());
        let result = service.delete_role(&non_existent_id).await;

        assert!(result.is_err(), "delete_role should have returned an error");

        match result {
            Err(error) => {
                assert!(
                    error.to_string().contains("not found"),
                    "Expected role not found error"
                );
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_role_does_not_affect_other_roles() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();

        let server_id = Uuid::new_v4();

        // Insert multiple roles
        let input1 = CreateRoleInput {
            server_id,
            name: "Role 1".to_string(),
            permissions: Permissions::try_from(0x1).unwrap(),
        };
        let role1 = service.create_role(input1).await?;

        let input2 = CreateRoleInput {
            server_id,
            name: "Role 2".to_string(),
            permissions: Permissions::try_from(0x2).unwrap(),
        };
        let role2 = service.create_role(input2).await?;

        // Delete role1
        service
            .delete_role(&role1.id)
            .await
            .expect("delete_role returned an error");

        // Verify role1 is deleted
        let result1 = service.role_repository.find_by_id(&role1.id).await;
        assert!(result1.is_err(), "Expected role 1 to be deleted");

        // Verify role2 still exists
        let existing_role2 = service
            .get_role(&role2.id)
            .await
            .expect("role 2 should still exist");
        assert_eq!(existing_role2.id, role2.id, "Expected role 2 to exist");
        assert_eq!(existing_role2.name, "Role 2", "Expected role 2 name");

        Ok(())
    }
}
