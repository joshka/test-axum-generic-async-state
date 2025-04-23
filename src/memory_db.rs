use std::collections::HashMap;
use std::sync::Arc;

use crate::items::Item;
use crate::users::User;

#[derive(Clone)]
pub struct MemoryDb {
    pub users: Arc<HashMap<u32, User>>,
    pub items: Arc<HashMap<u32, Item>>,
}

impl MemoryDb {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User::new(1, "foo".to_string()));
        users.insert(2, User::new(2, "bar".to_string()));

        let mut items = HashMap::new();
        items.insert(1, Item::new(1, "item1".to_string()));
        items.insert(2, Item::new(2, "item2".to_string()));

        Self {
            users: Arc::new(users),
            items: Arc::new(items),
        }
    }
}
