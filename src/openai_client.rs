use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse,
    },
    Client,
};
use teloxide::types::Message;
use tracing::instrument;

use crate::telegram_bot;

#[instrument]
pub async fn group_question(
    messages: &[Message],
    question: String,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let client = Client::new();

    let system_message = ChatCompletionRequestMessage {
        role: async_openai::types::Role::System,
        content: "You are a Telegram chat bot that helps humans to understand what is happening or has happened in group chats"
            .to_string(),
        name: None,
    };

    let mut chat_history = String::new();

    for message in messages {
        let username = message.from().and_then(|user| user.username.clone());
        let message_text = message.text();
        let message_time = message.date.naive_local();

        if let (Some(username), Some(message_text)) = (username, message_text) {
            chat_history
                .push_str(format!("{} [{}]: {}\n", username, message_time, message_text).as_str())
        }
    }

    let task_message = ChatCompletionRequestMessage {
        role: async_openai::types::Role::User,
        content: format!(
            "Use the following conversation as context: \n\n ###{}###  \n\n {} ",
            chat_history, question
        ),
        name: None,
    };

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-4")
        .messages(vec![system_message, task_message])
        .build()?;

    client.chat().create(request).await
}

impl From<telegram_bot::Role> for async_openai::types::Role {
    fn from(val: telegram_bot::Role) -> Self {
        match val {
            telegram_bot::Role::System => async_openai::types::Role::System,
            telegram_bot::Role::User => async_openai::types::Role::User,
            telegram_bot::Role::Assistant => async_openai::types::Role::Assistant,
        }
    }
}

impl From<telegram_bot::BotMessage> for async_openai::types::ChatCompletionRequestMessage {
    fn from(value: telegram_bot::BotMessage) -> Self {
        let mut req = ChatCompletionRequestMessageArgs::default();

        req.role(value.role).content(&value.content);

        if let Some(name) = &value.name {
            req.name(name);
        }

        req.build().unwrap()
    }
}

#[instrument]
pub async fn reply(
    messages: &[ChatCompletionRequestMessage],
    system: Option<&str>,
    model: Option<&str>,
) -> Result<CreateChatCompletionResponse, OpenAIError> {
    let client = Client::new();

    let system_msg = ChatCompletionRequestMessage {
        role: async_openai::types::Role::System,
        content: system
            .unwrap_or("You are GTP-4 a Telegram chat bot")
            .to_string(),
        name: None,
    };

    let mut request_messages = vec![system_msg];

    request_messages.extend_from_slice(messages);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model(model.unwrap_or("gpt-4"))
        .messages(messages)
        .build()?;

    client.chat().create(request).await
}