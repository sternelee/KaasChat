use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart,
        ChatCompletionRequestMessageContentPartImageArgs,
        ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, ImageDetail,
        ImageUrlArgs,
    },
};
use entity::entities::{
    contents::ContentType,
    messages::{MessageDTO, Roles},
    settings::ProxySetting,
};

use crate::{log_utils::warn, services::cache};

use super::providers::google::chat::{GoogleChatCompletionContent, GoogleChatCompletionContentPart, GoogleChatCompletionContentPartFileData, GoogleRole};

pub fn sum_option(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        _ => None,
    }
}

pub fn message_to_openai_request_message(message: MessageDTO) -> ChatCompletionRequestMessage {
    let log_tag = "utils::message_to_openai_request_message";
    match message.role.into() {
        Roles::User => {
            let content_parts = message
                .clone()
                .content
                .into_iter()
                .map(|item| {
                    let part: ChatCompletionRequestMessageContentPart = match item.r#type {
                        ContentType::Image => {
                            ChatCompletionRequestMessageContentPartImageArgs::default()
                                .image_url(
                                    ImageUrlArgs::default()
                                        .url(
                                            cache::read_as_data_url(
                                                item.data.as_str(),
                                                item.mimetype.as_deref(),
                                            )
                                            .unwrap_or(String::default()),
                                        )
                                        .detail(ImageDetail::Auto)
                                        .build()?,
                                )
                                .build()?
                                .into()
                        }
                        ContentType::Text => {
                            ChatCompletionRequestMessageContentPartTextArgs::default()
                                .text(item.data)
                                .build()?
                                .into()
                        }
                    };
                    Ok(part)
                })
                .collect::<Result<Vec<ChatCompletionRequestMessageContentPart>, OpenAIError>>()
                .expect("Failed to build user message content");
            if content_parts.len() == 1 {
                if let Some(part) = content_parts.first() {
                    match part {
                        ChatCompletionRequestMessageContentPart::Text(text) => {
                            return ChatCompletionRequestMessage::User(
                                ChatCompletionRequestUserMessage {
                                    content: ChatCompletionRequestUserMessageContent::Text(
                                        text.text.to_string(),
                                    ),
                                    name: None,
                                },
                            );
                        }
                        _ => {
                            warn(
                                log_tag,
                                format!("User message contains only image content: {:?}", message),
                            );
                        }
                    }
                }
            }

            return ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                name: None,
            });
        }
        Roles::System => {
            return ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: message.get_text().unwrap_or(String::default()),
                name: None,
            });
        }
        _ => {
            return ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(message.get_text().unwrap_or(String::default()))
                    .build()
                    .unwrap_or(ChatCompletionRequestAssistantMessage::default()),
            );
        }
    }
}

pub fn message_to_google_request_message(message: MessageDTO) -> GoogleChatCompletionContent {
    let content_parts = message
        .clone()
        .content
        .into_iter()
        .map(|item| {
            let part: GoogleChatCompletionContentPart = match item.r#type {
                ContentType::Image => {
                    let mime_type = item.mimetype.unwrap_or(String::default());
                    GoogleChatCompletionContentPart::FileData(GoogleChatCompletionContentPartFileData {
                        mime_type: mime_type.clone(),
                        file_uri: cache::read_as_data_url(item.data.as_str(), Some(&mime_type)).unwrap_or(String::default()),
                    })
                },
                ContentType::Text => {
                    GoogleChatCompletionContentPart::Text(item.data)
                }
            };
            Ok(part)
        }).collect::<Result<Vec<GoogleChatCompletionContentPart>, OpenAIError>>()
        .expect("Failed to build user message content");
    match message.role.into() {
        Roles::User => {
            GoogleChatCompletionContent {
                parts: Some(content_parts),
                role: GoogleRole::User,
            }
        },
        _ => {
            GoogleChatCompletionContent {
                parts: Some(content_parts),
                role: GoogleRole::Model,
            }
        }
    }
}

/// Build reqwest client with proxy
pub fn build_http_client(proxy_setting: Option<ProxySetting>) -> reqwest::Client {
    let proxy_option: Option<reqwest::Proxy> = if let Some(setting) = proxy_setting {
        if setting.on {
            let proxy_option = if setting.http && setting.https {
                reqwest::Proxy::all(setting.server).ok()
            } else if setting.http {
                reqwest::Proxy::http(setting.server).ok()
            } else {
                reqwest::Proxy::https(setting.server).ok()
            };
            if let Some(proxy) = proxy_option {
                if let (Some(username), Some(password)) = (setting.username, setting.password) {
                    Some(proxy.basic_auth(username.as_str(), password.as_str()))
                } else {
                    Some(proxy.to_owned())
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let mut http_client_builder = reqwest::Client::builder();
    if let Some(proxy) = proxy_option {
        http_client_builder = http_client_builder.proxy(proxy);
    }
    http_client_builder
        .build()
        .unwrap_or(reqwest::Client::new())
}
