use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub content: String,
    pub airport_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageBoard {
    messages: Vec<Message>,
    max_messages: usize,
}

impl MessageBoard {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
        }
    }

    pub fn post_message(
        &mut self,
        author_id: Uuid,
        author_name: String,
        content: String,
        airport_id: String,
    ) -> Result<Message, String> {
        if content.is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if content.len() > 500 {
            return Err("Message content cannot exceed 500 characters".to_string());
        }

        let message = Message {
            id: Uuid::new_v4(),
            author_id,
            author_name,
            content,
            airport_id,
            created_at: chrono::Utc::now(),
        };

        self.messages.push(message.clone());

        // Keep only the most recent messages if we exceed the limit
        if self.messages.len() > self.max_messages {
            self.messages
                .drain(0..self.messages.len() - self.max_messages);
        }

        Ok(message)
    }

    pub fn get_messages(&self, airport_id: &str, limit: Option<usize>) -> Vec<&Message> {
        let mut messages: Vec<&Message> = self
            .messages
            .iter()
            .filter(|msg| msg.airport_id == airport_id)
            .collect();

        // Sort by creation time (most recent first)
        messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            messages.truncate(limit);
        }

        messages
    }

    #[allow(dead_code)]
    pub fn get_all_messages(&self, limit: Option<usize>) -> Vec<&Message> {
        let mut messages: Vec<&Message> = self.messages.iter().collect();
        messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = limit {
            messages.truncate(limit);
        }

        messages
    }

    #[allow(dead_code)]
    pub fn clear_airport_messages(&mut self, airport_id: &str) {
        self.messages.retain(|msg| msg.airport_id != airport_id);
    }

    pub fn message_count(&self, airport_id: Option<&str>) -> usize {
        match airport_id {
            Some(id) => self
                .messages
                .iter()
                .filter(|msg| msg.airport_id == id)
                .count(),
            None => self.messages.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_and_get_messages() {
        let mut board = MessageBoard::new(100);
        let author_id = Uuid::new_v4();

        let message = board
            .post_message(
                author_id,
                "TestPlayer".to_string(),
                "Hello world!".to_string(),
                "JFK".to_string(),
            )
            .unwrap();

        assert_eq!(message.content, "Hello world!");
        assert_eq!(message.airport_id, "JFK");

        let messages = board.get_messages("JFK", None);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello world!");
    }

    #[test]
    fn test_message_limit() {
        let mut board = MessageBoard::new(3);
        let author_id = Uuid::new_v4();

        for i in 1..=5 {
            board
                .post_message(
                    author_id,
                    "TestPlayer".to_string(),
                    format!("Message {}", i),
                    "JFK".to_string(),
                )
                .unwrap();
        }

        assert_eq!(board.message_count(None), 3);
        let messages = board.get_messages("JFK", None);
        assert_eq!(messages[0].content, "Message 5");
    }

    #[test]
    fn test_empty_message_error() {
        let mut board = MessageBoard::new(100);
        let author_id = Uuid::new_v4();

        let result = board.post_message(
            author_id,
            "TestPlayer".to_string(),
            "".to_string(),
            "JFK".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Message content cannot be empty");
    }

    #[test]
    fn test_long_message_error() {
        let mut board = MessageBoard::new(100);
        let author_id = Uuid::new_v4();
        let long_content = "a".repeat(501);

        let result = board.post_message(
            author_id,
            "TestPlayer".to_string(),
            long_content,
            "JFK".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Message content cannot exceed 500 characters"
        );
    }
}
