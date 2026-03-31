//! Conversation helper — manages chat history automatically

use crate::error::Result;
use crate::http_client::HttpClient;
use crate::http_types::{ChatMessage, ChatRequest};

const DEFAULT_MODEL: &str = "default";

/// Stateful conversation that accumulates chat history across turns
pub struct Conversation {
    client: HttpClient,
    history: Vec<ChatMessage>,
    model: String,
}

impl Conversation {
    /// Create a new conversation using the given HTTP client
    ///
    /// # Example
    /// ```rust,no_run
    /// use wauldo::{HttpClient, Conversation};
    /// let client = HttpClient::localhost().unwrap();
    /// let conv = Conversation::new(client);
    /// ```
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
            history: Vec::new(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Add a system message to the conversation history
    ///
    /// The system message is always inserted at position 0 so it precedes
    /// all user and assistant turns.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{HttpClient, Conversation};
    /// # let client = HttpClient::localhost().unwrap();
    /// let conv = Conversation::new(client)
    ///     .with_system("You are a helpful Rust expert");
    /// ```
    pub fn with_system(mut self, system: &str) -> Self {
        // Replace existing system message if present, otherwise insert at start
        if self.history.first().map(|m| m.role.as_str()) == Some("system") {
            self.history[0] = ChatMessage::system(system);
        } else {
            self.history.insert(0, ChatMessage::system(system));
        }
        self
    }

    /// Set the model to use for chat completions
    ///
    /// Defaults to `"default"` (server-selected) if not called.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{HttpClient, Conversation};
    /// # let client = HttpClient::localhost().unwrap();
    /// let conv = Conversation::new(client).with_model("llama3");
    /// ```
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Send a user message, receive the assistant reply, and store both in history
    ///
    /// The full accumulated history (system + prior turns) is sent with each
    /// request so the LLM has complete conversational context.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{HttpClient, Conversation};
    /// # async fn run() -> wauldo::Result<()> {
    /// # let client = HttpClient::localhost()?;
    /// let mut conv = Conversation::new(client).with_system("Be concise");
    /// let reply = conv.say("What is Rust?").await?;
    /// println!("{}", reply);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn say(&mut self, message: &str) -> Result<String> {
        let rollback_len = self.history.len();
        self.history.push(ChatMessage::user(message));

        let req = ChatRequest::new(self.model.clone(), self.history.clone());
        let resp = match self.client.chat(req).await {
            Ok(r) => r,
            Err(e) => {
                // Rollback: remove the user message on error to avoid history corruption
                self.history.truncate(rollback_len);
                return Err(e);
            }
        };

        let content = resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        self.history.push(ChatMessage::assistant(&content));
        Ok(content)
    }

    /// Read-only access to the accumulated history
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{HttpClient, Conversation};
    /// # let client = HttpClient::localhost().unwrap();
    /// let conv = Conversation::new(client).with_system("sys");
    /// assert_eq!(conv.history().len(), 1);
    /// ```
    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    /// Clear conversation history (keeps model setting and system prompt)
    ///
    /// Removes all user and assistant messages but preserves the system prompt
    /// (if any) so subsequent `say()` calls retain the same persona.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use wauldo::{HttpClient, Conversation};
    /// # let client = HttpClient::localhost().unwrap();
    /// let mut conv = Conversation::new(client).with_system("sys");
    /// conv.clear();
    /// assert_eq!(conv.history().len(), 1); // system prompt preserved
    /// ```
    pub fn clear(&mut self) {
        let system = self
            .history
            .first()
            .filter(|m| m.role == "system")
            .cloned();
        self.history.clear();
        if let Some(sys) = system {
            self.history.push(sys);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_with_system_inserts_at_start() {
        let client = HttpClient::localhost().unwrap();
        let conv = Conversation::new(client)
            .with_system("You are helpful")
            .with_model("test-model");

        assert_eq!(conv.history().len(), 1);
        assert_eq!(conv.history()[0].role, "system");
        assert_eq!(
            conv.history()[0].content.as_deref(),
            Some("You are helpful")
        );
    }

    #[test]
    fn test_conversation_clear_preserves_system() {
        let client = HttpClient::localhost().unwrap();
        let mut conv = Conversation::new(client).with_system("sys");
        assert_eq!(conv.history().len(), 1);
        conv.clear();
        assert_eq!(conv.history().len(), 1);
        assert_eq!(conv.history()[0].role, "system");
    }

    #[test]
    fn test_conversation_clear_without_system() {
        let client = HttpClient::localhost().unwrap();
        let mut conv = Conversation::new(client);
        conv.clear();
        assert!(conv.history().is_empty());
    }
}
