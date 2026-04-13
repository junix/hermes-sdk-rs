use crate::types::{CreateResponseRequest, Input, InputMessage};

/// Fluent builder for [`CreateResponseRequest`].
#[derive(Debug, Clone, Default)]
pub struct CreateResponseRequestBuilder {
    input: Option<Input>,
    instructions: Option<String>,
    previous_response_id: Option<String>,
    conversation: Option<String>,
    store: Option<bool>,
    model: Option<String>,
    conversation_history: Option<Vec<InputMessage>>,
}

impl CreateResponseRequestBuilder {
    /// Set the input as a simple text string.
    pub fn input(mut self, text: impl Into<String>) -> Self {
        self.input = Some(Input::Text(text.into()));
        self
    }

    /// Set the input as structured messages.
    pub fn messages(mut self, messages: Vec<InputMessage>) -> Self {
        self.input = Some(Input::Messages(messages));
        self
    }

    /// System-level instructions for this turn.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Chain to a previous response by ID.
    ///
    /// Mutually exclusive with `conversation`.
    pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.previous_response_id = Some(id.into());
        self
    }

    /// Use a named conversation (auto-chains to latest response).
    ///
    /// Mutually exclusive with `previous_response_id`.
    pub fn conversation(mut self, name: impl Into<String>) -> Self {
        self.conversation = Some(name.into());
        self
    }

    /// Whether to store the response for later retrieval (default: true).
    pub fn store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Override the model name.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Provide explicit conversation history.
    pub fn conversation_history(mut self, history: Vec<InputMessage>) -> Self {
        self.conversation_history = Some(history);
        self
    }

    /// Build the request, validating that constraints are met.
    pub fn build(self) -> Result<CreateResponseRequest, crate::types::HermesError> {
        let input = self
            .input
            .ok_or_else(|| crate::types::HermesError::Config("input is required".into()))?;

        if self.conversation.is_some() && self.previous_response_id.is_some() {
            return Err(crate::types::HermesError::Config(
                "conversation and previous_response_id are mutually exclusive".into(),
            ));
        }

        Ok(CreateResponseRequest {
            input,
            instructions: self.instructions,
            previous_response_id: self.previous_response_id,
            conversation: self.conversation,
            store: self.store,
            model: self.model,
            conversation_history: self.conversation_history,
        })
    }
}
