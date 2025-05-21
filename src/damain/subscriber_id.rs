#[derive(Debug)]
pub struct SubscriberId(uuid::Uuid);

impl SubscriberId {
    pub fn new(id: uuid::Uuid) -> Self {
        Self(id)
    }

    pub fn inner(self) -> uuid::Uuid {
        self.0
    }
}
