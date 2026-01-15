use sqlx::{PgPool, query_as};

use crate::{
    domain::{
        common::CoreError,
        member_role::{
            entities::{AssignMemberRole, MemberRole, UnassignMemberRole},
            ports::MemberRoleRepository,
        },
    },
    infrastructure::{MessageRoutingInfo, outbox::OutboxEventRecord},
};

pub struct PostgresMemberRoleRepository {
    pool: PgPool,
    assign_role_routing: MessageRoutingInfo,
}

impl PostgresMemberRoleRepository {
    pub fn new(pool: PgPool, assign_role_routing: MessageRoutingInfo) -> Self {
        Self {
            pool,
            assign_role_routing,
        }
    }
}

impl MemberRoleRepository for PostgresMemberRoleRepository {
    async fn assign(&self, member_role: AssignMemberRole) -> Result<MemberRole, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Error { msg: e.to_string() })?;

        let member_role = query_as!(
            MemberRole,
            r#"
            INSERT INTO member_roles (role_id, member_id)
            VALUES ($1, $2)
            RETURNING role_id, member_id, created_at, updated_at
            "#,
            *member_role.role_id,
            *member_role.member_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| CoreError::AssignMemberRoleError {
            member_id: member_role.member_id,
            role_id: member_role.role_id,
        })?;

        let assign_member_to_role_event =
            OutboxEventRecord::new(self.assign_role_routing.clone(), member_role.clone());

        assign_member_to_role_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::Error { msg: e.to_string() })?;

        Ok(member_role)
    }

    async fn unassign(&self, member_role: UnassignMemberRole) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Error { msg: e.to_string() })?;

        sqlx::query(r#"DELETE FROM member_roles WHERE member_id = $1 AND role_id = $2"#)
            .bind(*member_role.member_id)
            .bind(*member_role.role_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::DatabaseError { msg: e.to_string() })?;

        let unassign_member_from_role_event =
            OutboxEventRecord::new(self.assign_role_routing.clone(), member_role.clone());

        unassign_member_from_role_event.write(&mut *tx).await?;

        tx.commit()
            .await
            .map_err(|e| CoreError::Error { msg: e.to_string() })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{PgPool, query_as};
    use uuid::Uuid;

    use crate::{
        domain::{
            common::CoreError,
            member_role::{
                entities::{AssignMemberRole, UnassignMemberRole},
                ports::MemberRoleRepository,
            },
            role::entities::{Permissions, Role, RoleId},
            server::entities::ServerVisibility,
            server_member::{MemberId, ServerMember},
        },
        infrastructure::{
            MessageRoutingInfo, member_role::repositories::postgres::PostgresMemberRoleRepository,
        },
    };
    pub async fn create_test_member(pool: &PgPool, server_id: Uuid) -> Uuid {
        let row = sqlx::query(
            r#"
            INSERT INTO server_members (id, server_id, user_id)
            VALUES ($1, $2, $3)
            RETURNING id, server_id, user_id, nickname, joined_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(server_id)
        .bind(Uuid::new_v4())
        .fetch_one(pool)
        .await
        .unwrap();

        let member: ServerMember = (&row).into();
        *member.id
    }

    async fn create_test_role(pool: &PgPool, server_id: Uuid) -> Uuid {
        let permission = Permissions(0x1);

        let role = query_as!(
            Role,
            r#"
            INSERT INTO roles (server_id, name, permissions)
            VALUES ($1, $2, $3)
            RETURNING id, server_id, name, permissions as "permissions: _", created_at, updated_at
            "#,
            server_id,
            "default_test_role",
            *permission
        )
        .fetch_one(pool)
        .await
        .unwrap();
        *role.id
    }

    async fn create_test_server(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO servers (name, owner_id, visibility)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(Uuid::new_v4())
        .bind(ServerVisibility::Private)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_assign_member_to_role(pool: PgPool) -> Result<(), CoreError> {
        let assign_role_routing =
            MessageRoutingInfo::new("test".to_string(), "test_routing".to_string());
        let repository = PostgresMemberRoleRepository::new(pool.clone(), assign_role_routing);
        let server_id = create_test_server(&pool.clone(), "test_server").await;
        let role_id = create_test_role(&pool.clone(), server_id).await;
        let member_id = create_test_member(&pool.clone(), server_id).await;
        let assign_member_role = AssignMemberRole {
            member_id: MemberId(member_id),
            role_id: RoleId(role_id),
        };
        let assigned = repository.assign(assign_member_role).await.unwrap();

        assert_eq!(*assigned.member_id, member_id);
        assert_eq!(assigned.role_id, RoleId(role_id));
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_unassign_member_from_role(pool: PgPool) -> Result<(), CoreError> {
        let assign_role_routing =
            MessageRoutingInfo::new("test".to_string(), "test_routing".to_string());
        let repository = PostgresMemberRoleRepository::new(pool.clone(), assign_role_routing);
        let server_id = create_test_server(&pool.clone(), "test_server").await;
        let role_id = create_test_role(&pool.clone(), server_id).await;
        let member_id = create_test_member(&pool.clone(), server_id).await;
        let assign_member_role = AssignMemberRole {
            member_id: MemberId(member_id),
            role_id: RoleId(role_id),
        };
        let _ = repository.assign(assign_member_role).await.unwrap();
        let unassign = UnassignMemberRole {
            member_id: MemberId(member_id),
            role_id: RoleId(role_id),
        };
        repository.unassign(unassign).await.unwrap();
        Ok(())
    }
}
