use uuid::{NoContext, Timestamp, Uuid};

pub fn generate_uuid() -> Uuid {
    let ts = Timestamp::now(NoContext);
    Uuid::new_v7(ts)
}