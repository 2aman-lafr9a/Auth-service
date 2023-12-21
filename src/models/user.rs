use diesel::prelude::*;
use crate::schema::users::dsl::users;
use crate::schema::users::{password_hash, role, username};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}


// Implement the necessary traits for the User struct
impl User {
    pub fn create_user(conn: &mut MysqlConnection, p_username: &str, p_password_hash: &str, p_role: &str) -> QueryResult<usize> {
        diesel::insert_into(users)
            .values((username.eq(p_username), password_hash.eq(p_password_hash), role.eq(p_role)))
            .execute(conn)
    }

    pub fn find_user_by_username(conn: &mut MysqlConnection, target_username: &str) -> QueryResult<Option<User>> {
        users.filter(username.eq(target_username))
            .first(conn)
            .optional()
    }
}
