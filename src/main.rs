/*
 * Copyright © 2023 Archer <archer@nefarious.dev>
 * Licensed under the Apache License, Version 2.0 (the "Licence");
 * you may not use this file except in compliance with the Licence.
 * You may obtain a copy of the Licence at
 *     https://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the Licence is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the Licence for the specific language governing permissions and
 * limitations under the Licence.
 */

use std::env;
use std::error::Error;
use std::sync::{Arc, OnceLock};
use twilight_gateway::Shard;
use twilight_http::Client;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::message::component::{
    ActionRow, ChannelSelectMenuData, SelectMenu, SelectMenuData, SelectMenuOption,
    TextSelectMenuData,
};
use twilight_model::channel::message::{AllowedMentions, Component, MessageFlags, ReactionType};
use twilight_model::channel::ChannelType;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{ApplicationMarker, ChannelMarker};
use twilight_model::id::Id;

static APP_ID: OnceLock<Id<ApplicationMarker>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Make it easier to verify that this bot doesn't do anything fishy with the token by moving it to a separate block.
    // Even if the token would "survive" the move in `Shard::new`, it would be dropped at the end of the block.
    let (client, mut shard) = {
        let token = env::var("DISCORD_TOKEN")?;
        let client = Arc::new(Client::new(token.clone()));
        let shard = Shard::new(
            ShardId::ONE,
            token,
            Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES,
        );
        (client, shard)
    };

    // Retrieve the application ID
    APP_ID
        .set(client.current_user_application().await?.model().await?.id)
        .unwrap();

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(why) => {
                if why.is_fatal() {
                    break;
                } else {
                    continue;
                }
            }
        };
        match event {
            // We don't use fancy application commands, as this is a simple test bot
            Event::MessageCreate(message) if message.content.eq_ignore_ascii_case("!select") => {
                let client = client.clone();
                tokio::spawn(async move { send_select_menus(client, message.channel_id).await });
            }
            Event::InteractionCreate(interaction)
                if interaction.kind == InteractionType::MessageComponent =>
            {
                let client = client.clone();
                tokio::spawn(async move { handle_select_menu(client, *interaction).await });
            }
            _ => {}
        }
    }

    Ok(())
}

async fn send_select_menus(client: Arc<Client>, channel: Id<ChannelMarker>) {
    static COMPONENTS: OnceLock<Vec<Component>> = OnceLock::new();
    let components = COMPONENTS.get_or_init(|| {
        vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(SelectMenu {
                    custom_id: String::from("text-select"),
                    data: SelectMenuData::Text(Box::new(TextSelectMenuData {
                        options: vec![
                            SelectMenuOption {
                                default: false,
                                value: String::from("foo"),
                                emoji: Some(ReactionType::Unicode {
                                    name: String::from("🐕"),
                                }),
                                label: String::from("Foo"),
                                description: Some(String::from("The foo")),
                            },
                            SelectMenuOption {
                                default: false,
                                value: String::from("bar"),
                                emoji: Some(ReactionType::Unicode {
                                    name: String::from("🦊"),
                                }),
                                label: String::from("Bar"),
                                description: Some(String::from("The bar")),
                            },
                            SelectMenuOption {
                                default: false,
                                value: String::from("baz"),
                                emoji: Some(ReactionType::Unicode {
                                    name: String::from("🐈"),
                                }),
                                label: String::from("Baz"),
                                description: Some(String::from("The baz")),
                            },
                        ],
                    })),
                    disabled: false,
                    max_values: None,
                    min_values: None,
                    placeholder: Some(String::from("Text select menu")),
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(SelectMenu {
                    custom_id: String::from("user-select"),
                    data: SelectMenuData::User,
                    disabled: false,
                    max_values: None,
                    min_values: None,
                    placeholder: Some(String::from("User select menu")),
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(SelectMenu {
                    custom_id: String::from("role-select"),
                    data: SelectMenuData::Role,
                    disabled: false,
                    max_values: None,
                    min_values: None,
                    placeholder: Some(String::from("Role select menu")),
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(SelectMenu {
                    custom_id: String::from("mentionable-select"),
                    data: SelectMenuData::Mentionable,
                    disabled: false,
                    max_values: None,
                    min_values: None,
                    placeholder: Some(String::from("Mentionable select menu")),
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(SelectMenu {
                    custom_id: String::from("channel-select"),
                    data: SelectMenuData::Channel(Box::new(ChannelSelectMenuData {
                        channel_types: Some(vec![
                            ChannelType::GuildText,
                            ChannelType::PublicThread,
                            ChannelType::PrivateThread,
                            ChannelType::GuildCategory,
                        ]),
                    })),
                    disabled: false,
                    max_values: None,
                    min_values: None,
                    placeholder: Some(String::from("Channel select menu")),
                })],
            }),
        ]
    });

    // Let's hope Discord doesn't add more components without increasing the action-rows-per-message limit Kappa

    let _ = client
        .create_message(channel)
        .content("Try using the select menus below")
        .components(components)
        .await;
}

async fn handle_select_menu(client: Arc<Client>, interaction: InteractionCreate) {
    let client = client.interaction(*APP_ID.get().unwrap());
    let Some(InteractionData::MessageComponent(data)) = &interaction.data else {
        return;
    };
    let _ = client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!(
                        "You interacted with a select menu!\n\
                         - Select menu ID: {}\n\
                         - Selected Values: `{:?}`\n\
                         - Resolved data: `{:?}`",
                        data.custom_id, data.values, data.resolved
                    )),
                    components: None,
                    custom_id: None,
                    flags: Some(MessageFlags::EPHEMERAL),
                    title: None,
                    allowed_mentions: Some(AllowedMentions::default()),
                    attachments: None,
                    choices: None,
                    embeds: None,
                    tts: Some(false),
                }),
            },
        )
        .await;
}
