use std::sync::Arc;

use gilvave_core::{
    dto::users::friends::*,
    error::{DatabaseError, users::friends::FriendError},
    ids::UserId,
};

use crate::db::Database;

#[inline]
fn canonical_pair(a: UserId, b: UserId) -> (UserId, UserId) {
    if a < b { (a, b) } else { (b, a) }
}

#[derive(Clone)]
pub struct FriendsService {
    pub db: Arc<Database>,
}

impl FriendsService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_friends(&self, user_id: UserId) -> Result<Vec<FriendView>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT 
                    u.id, 
                    u.username, 
                    COALESCE(p.name, u.username) AS display_name, 
                    COALESCE(p.avatar, '') AS avatar, 
                    f.created_at
                FROM friends f
                JOIN users u ON u.id = CASE WHEN f.user_id1 = $1 THEN f.user_id2 ELSE f.user_id1 END
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE (f.user_id1 = $1 OR f.user_id2 = $1)
                  AND NOT EXISTS (
                      SELECT 1 FROM user_blocks ub 
                      WHERE (ub.blocker_id = $1 AND ub.blocked_id = u.id)
                         OR (ub.blocker_id = u.id AND ub.blocked_id = $1)
                  )
                ORDER BY display_name ASC;
                "#,
                &[&user_id],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(FriendView::from_row(row)?);
        }
        Ok(list)
    }

    pub async fn get_requests(&self, user_id: UserId) -> Result<FriendRequestsList, FriendError> {
        let params: [&(dyn tokio_postgres::types::ToSql + Sync); 1] = [&user_id];

        let (incoming_rows, outgoing_rows) = tokio::try_join!(
            self.db.query(
                r#"
                SELECT 
                    u.id, 
                    u.username, 
                    COALESCE(p.name, u.username) AS display_name, 
                    COALESCE(p.avatar, '') AS avatar, 
                    fr.created_at
                FROM friend_requests fr
                JOIN users u ON u.id = fr.sender_id
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE fr.receiver_id = $1
                  AND NOT EXISTS (
                      SELECT 1 FROM user_blocks ub 
                      WHERE (ub.blocker_id = $1 AND ub.blocked_id = u.id)
                         OR (ub.blocker_id = u.id AND ub.blocked_id = $1)
                  )
                ORDER BY fr.created_at DESC;
                "#,
                &params,
            ),
            self.db.query(
                r#"
                SELECT 
                    u.id, 
                    u.username, 
                    COALESCE(p.name, u.username) AS display_name, 
                    COALESCE(p.avatar, '') AS avatar, 
                    fr.created_at
                FROM friend_requests fr
                JOIN users u ON u.id = fr.receiver_id
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE fr.sender_id = $1
                  AND NOT EXISTS (
                      SELECT 1 FROM user_blocks ub 
                      WHERE (ub.blocker_id = $1 AND ub.blocked_id = u.id)
                         OR (ub.blocker_id = u.id AND ub.blocked_id = $1)
                  )
                ORDER BY fr.created_at DESC;
                "#,
                &params,
            ),
        )?;

        let mut incoming = Vec::with_capacity(incoming_rows.len());
        for row in &incoming_rows {
            incoming.push(FriendRequestView::from_row(row)?);
        }

        let mut outgoing = Vec::with_capacity(outgoing_rows.len());
        for row in &outgoing_rows {
            outgoing.push(FriendRequestView::from_row(row)?);
        }

        Ok(FriendRequestsList { incoming, outgoing })
    }

    pub async fn send_request(
        &self,
        sender_id: UserId,
        req: SendFriendRequest,
    ) -> Result<SendFriendResponse, FriendError> {
        let target_id = if let Some(id) = req.user_id {
            id
        } else if let Some(username) = req.username.as_deref() {
            let row = self
                .db
                .query_opt("SELECT id FROM users WHERE username = $1;", &[&username])
                .await?;
            let Some(row) = row else {
                return Err(FriendError::UserNotFound);
            };
            row.try_get(0)?
        } else {
            return Err(FriendError::UserNotFound);
        };

        if sender_id == target_id {
            return Err(FriendError::SelfRequest);
        }

        let (u1, u2) = canonical_pair(sender_id, target_id);
        let params_target: [&(dyn tokio_postgres::types::ToSql + Sync); 1] = [&target_id];
        let params_pair: [&(dyn tokio_postgres::types::ToSql + Sync); 2] = [&sender_id, &target_id];
        let params_rev: [&(dyn tokio_postgres::types::ToSql + Sync); 2] = [&target_id, &sender_id];
        let params_canon: [&(dyn tokio_postgres::types::ToSql + Sync); 2] = [&u1, &u2];

        let (user_exists, blocked, already_friends, mutual_request, existing_request) = tokio::try_join!(
            self.db
                .query_opt("SELECT 1 FROM users WHERE id = $1;", &params_target),
            self.db.query_opt(
                r#"
                SELECT 1 FROM user_blocks 
                WHERE (blocker_id = $1 AND blocked_id = $2)
                   OR (blocker_id = $2 AND blocked_id = $1);
                "#,
                &params_pair,
            ),
            self.db.query_opt(
                "SELECT 1 FROM friends WHERE user_id1 = $1 AND user_id2 = $2;",
                &params_canon,
            ),
            self.db.query_opt(
                "SELECT 1 FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2;",
                &params_rev,
            ),
            self.db.query_opt(
                "SELECT 1 FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2;",
                &params_pair,
            ),
        )?;

        if user_exists.is_none() {
            return Err(FriendError::UserNotFound);
        }

        if blocked.is_some() {
            return Err(FriendError::Blocked);
        }

        if already_friends.is_some() {
            return Err(FriendError::AlreadyFriends);
        }

        if mutual_request.is_some() {
            let mut client = self.db.get_client().await?;
            let tx = client.transaction().await?;

            tx.execute(
                "DELETE FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2;",
                &params_rev,
            )
            .await?;

            tx.execute(
                "INSERT INTO friends (user_id1, user_id2) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
                &params_canon,
            )
            .await?;

            tx.commit().await?;

            return Ok(SendFriendResponse {
                status: FriendRequestResult::Accepted,
                message: "Взаимное приглашение обнаружено: пользователь добавлен в друзья"
                    .to_string(),
            });
        }

        if existing_request.is_some() {
            return Err(FriendError::RequestAlreadyExists);
        }

        self.db
            .execute(
                "INSERT INTO friend_requests (sender_id, receiver_id) VALUES ($1, $2);",
                &params_pair,
            )
            .await?;

        Ok(SendFriendResponse {
            status: FriendRequestResult::Sent,
            message: "Запрос в друзья успешно отправлен".to_string(),
        })
    }

    pub async fn accept_request(
        &self,
        receiver_id: UserId,
        sender_id: UserId,
    ) -> Result<(), FriendError> {
        if receiver_id == sender_id {
            return Err(FriendError::SelfRequest);
        }

        let blocked = self
            .db
            .query_opt(
                r#"
                SELECT 1 FROM user_blocks 
                WHERE (blocker_id = $1 AND blocked_id = $2)
                   OR (blocker_id = $2 AND blocked_id = $1);
                "#,
                &[&receiver_id, &sender_id],
            )
            .await?;

        if blocked.is_some() {
            return Err(FriendError::Blocked);
        }

        let mut client = self.db.get_client().await?;
        let tx = client.transaction().await?;

        let deleted = tx
            .execute(
                "DELETE FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2;",
                &[&sender_id, &receiver_id],
            )
            .await?;

        if deleted == 0 {
            return Err(FriendError::RequestNotFound);
        }

        let (u1, u2) = canonical_pair(receiver_id, sender_id);
        tx.execute(
            "INSERT INTO friends (user_id1, user_id2) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            &[&u1, &u2],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn decline_request(
        &self,
        user_id: UserId,
        other_user_id: UserId,
    ) -> Result<(), FriendError> {
        let deleted = self
            .db
            .execute(
                r#"
                DELETE FROM friend_requests 
                WHERE (sender_id = $1 AND receiver_id = $2)
                   OR (sender_id = $2 AND receiver_id = $1);
                "#,
                &[&user_id, &other_user_id],
            )
            .await?;

        if deleted == 0 {
            return Err(FriendError::RequestNotFound);
        }

        Ok(())
    }

    pub async fn remove_friend(
        &self,
        user_id: UserId,
        friend_id: UserId,
    ) -> Result<(), FriendError> {
        let (u1, u2) = canonical_pair(user_id, friend_id);
        self.db
            .execute(
                "DELETE FROM friends WHERE user_id1 = $1 AND user_id2 = $2;",
                &[&u1, &u2],
            )
            .await?;
        Ok(())
    }

    pub async fn block_user(
        &self,
        blocker_id: UserId,
        blocked_id: UserId,
    ) -> Result<(), FriendError> {
        if blocker_id == blocked_id {
            return Err(FriendError::SelfBlock);
        }

        let exists = self
            .db
            .query_opt("SELECT 1 FROM users WHERE id = $1;", &[&blocked_id])
            .await?;
        if exists.is_none() {
            return Err(FriendError::UserNotFound);
        }

        let (u1, u2) = canonical_pair(blocker_id, blocked_id);
        let mut client = self.db.get_client().await?;
        let tx = client.transaction().await?;

        tx.execute(
            "INSERT INTO user_blocks (blocker_id, blocked_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            &[&blocker_id, &blocked_id],
        )
        .await?;

        tx.execute(
            "DELETE FROM friends WHERE user_id1 = $1 AND user_id2 = $2;",
            &[&u1, &u2],
        )
        .await?;

        tx.execute(
            r#"
            DELETE FROM friend_requests 
            WHERE (sender_id = $1 AND receiver_id = $2)
               OR (sender_id = $2 AND receiver_id = $1);
            "#,
            &[&blocker_id, &blocked_id],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn unblock_user(
        &self,
        blocker_id: UserId,
        blocked_id: UserId,
    ) -> Result<(), FriendError> {
        self.db
            .execute(
                "DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2;",
                &[&blocker_id, &blocked_id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_blocked(
        &self,
        blocker_id: UserId,
    ) -> Result<Vec<BlockedUserView>, DatabaseError> {
        let rows = self
            .db
            .query(
                r#"
                SELECT 
                    u.id, 
                    u.username, 
                    COALESCE(p.name, u.username) AS display_name, 
                    COALESCE(p.avatar, '') AS avatar
                FROM user_blocks ub
                JOIN users u ON u.id = ub.blocked_id
                LEFT JOIN profile p ON p.user_id = u.id
                WHERE ub.blocker_id = $1
                ORDER BY display_name ASC;
                "#,
                &[&blocker_id],
            )
            .await?;

        let mut list = Vec::with_capacity(rows.len());
        for row in &rows {
            list.push(BlockedUserView::from_row(row)?);
        }
        Ok(list)
    }
}
