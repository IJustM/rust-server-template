use time::OffsetDateTime;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: OffsetDateTime,
}
