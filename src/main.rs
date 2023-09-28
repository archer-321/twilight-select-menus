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

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::sync::{Arc, OnceLock};
use twilight_gateway::Shard;
use twilight_http::client::InteractionClient;
use twilight_http::Client;
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::message::component::{
    ActionRow, SelectDefaultValue, SelectMenu, SelectMenuOption, SelectMenuType,
};
use twilight_model::channel::message::{AllowedMentions, Component, MessageFlags, ReactionType};
use twilight_model::channel::{Channel, ChannelType};
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::{
    ApplicationMarker, ChannelMarker, InteractionMarker, RoleMarker, UserMarker,
};
use twilight_model::id::Id;
use twilight_util::builder::command::{ChannelBuilder, CommandBuilder, RoleBuilder, UserBuilder};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

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

    // Register our application command
    client
        .interaction(*APP_ID.get().unwrap())
        .set_global_commands(&[CommandBuilder::new(
            "select-menu",
            "Send a list of test select menus",
            CommandType::ChatInput,
        )
        .dm_permission(false)
        .option(
            UserBuilder::new(
                "user-1",
                "A default value for the user and mentionable select menus",
            )
            .build(),
        )
        .option(
            UserBuilder::new(
                "user-2",
                "A default value for the user and mentionable select menus",
            )
            .build(),
        )
        .option(
            RoleBuilder::new(
                "role-1",
                "A default value for the role and mentionable select mneus",
            )
            .build(),
        )
        .option(
            RoleBuilder::new(
                "role-2",
                "A default value for the role and mentionable select mneus",
            )
            .build(),
        )
        .option(
            ChannelBuilder::new("channel-1", "A default value for the channel select menu")
                .channel_types([
                    ChannelType::GuildText,
                    ChannelType::PublicThread,
                    ChannelType::PrivateThread,
                    ChannelType::GuildCategory,
                ])
                .build(),
        )
        .option(
            ChannelBuilder::new("channel-2", "A default value for the channel select menu")
                .channel_types([
                    ChannelType::GuildText,
                    ChannelType::PublicThread,
                    ChannelType::PrivateThread,
                    ChannelType::GuildCategory,
                ])
                .build(),
        )
        .build()])
        .await?;

    'event_loop: loop {
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
            Event::InteractionCreate(interaction)
                if interaction.kind == InteractionType::ApplicationCommand =>
            {
                if let Some(InteractionData::ApplicationCommand(data)) = interaction.0.data {
                    if data.name == "select-menu" {
                        let (mut users, mut roles, mut channels) =
                            (HashSet::new(), HashSet::new(), HashSet::new());
                        let client = client.clone();
                        let interaction_id = interaction.0.id;
                        let token = interaction.0.token;
                        for option in data.options {
                            if option.name.starts_with("user-") {
                                let CommandOptionValue::User(user) = option.value else {
                                    tokio::spawn(async move {
                                        report_error(
                                            client.interaction(*APP_ID.get().unwrap()),
                                            interaction_id,
                                            &token,
                                        )
                                        .await;
                                    });
                                    continue 'event_loop;
                                };
                                users.insert(user);
                            } else if option.name.starts_with("role-") {
                                let CommandOptionValue::Role(role) = option.value else {
                                    tokio::spawn(async move {
                                        report_error(
                                            client.interaction(*APP_ID.get().unwrap()),
                                            interaction_id,
                                            &token,
                                        )
                                        .await;
                                    });
                                    continue 'event_loop;
                                };
                                roles.insert(role);
                            } else if option.name.starts_with("channel-") {
                                let CommandOptionValue::Channel(channel) = option.value else {
                                    tokio::spawn(async move {
                                        report_error(
                                            client.interaction(*APP_ID.get().unwrap()),
                                            interaction_id,
                                            &token,
                                        )
                                        .await;
                                    });
                                    continue 'event_loop;
                                };
                                channels.insert(channel);
                            }
                        }
                        let Some(Channel { id: channel_id, .. }) = interaction.0.channel else {
                            tokio::spawn(async move {
                                report_error(
                                    client.interaction(*APP_ID.get().unwrap()),
                                    interaction_id,
                                    &token,
                                )
                                .await;
                            });
                            continue;
                        };
                        tokio::spawn(async move {
                            let _ = client
                                .interaction(*APP_ID.get().unwrap())
                                .create_response(
                                    interaction_id,
                                    &token,
                                    &InteractionResponse {
                                        kind: InteractionResponseType::ChannelMessageWithSource,
                                        data: Some(
                                            InteractionResponseDataBuilder::new()
                                                .content("Sending a message with select menus...")
                                                .flags(MessageFlags::EPHEMERAL)
                                                .build(),
                                        ),
                                    },
                                )
                                .await;
                            send_select_menus(
                                client,
                                channel_id,
                                Vec::from_iter(users),
                                Vec::from_iter(roles),
                                Vec::from_iter(channels),
                            )
                            .await;
                        });
                    }
                }
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

async fn send_select_menus(
    client: Arc<Client>,
    channel: Id<ChannelMarker>,
    users: Vec<Id<UserMarker>>,
    roles: Vec<Id<RoleMarker>>,
    channels: Vec<Id<ChannelMarker>>,
) {
    let components = vec![
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                channel_types: None,
                custom_id: String::from("text-select"),
                default_values: None,
                disabled: false,
                kind: SelectMenuType::Text,
                max_values: None,
                min_values: None,
                options: Some(vec![
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
                ]),
                placeholder: Some(String::from("Text select menu")),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                channel_types: None,
                custom_id: String::from("user-select"),
                default_values: if users.is_empty() {
                    None
                } else {
                    Some(
                        users
                            .iter()
                            .map(|id| SelectDefaultValue::User(*id))
                            .collect(),
                    )
                },
                disabled: false,
                kind: SelectMenuType::User,
                max_values: Some(5),
                min_values: None,
                options: None,
                placeholder: Some(String::from("User select menu")),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                channel_types: None,
                custom_id: String::from("role-select"),
                default_values: if roles.is_empty() {
                    None
                } else {
                    Some(
                        roles
                            .iter()
                            .map(|id| SelectDefaultValue::Role(*id))
                            .collect(),
                    )
                },
                disabled: false,
                kind: SelectMenuType::Role,
                max_values: Some(5),
                min_values: None,
                options: None,
                placeholder: Some(String::from("Role select menu")),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                channel_types: None,
                custom_id: String::from("mentionable-select"),
                default_values: if users.is_empty() && roles.is_empty() {
                    None
                } else {
                    Some(
                        users
                            .iter()
                            .map(|id| SelectDefaultValue::User(*id))
                            .chain(roles.iter().map(|id| SelectDefaultValue::Role(*id)))
                            .collect(),
                    )
                },
                disabled: false,
                kind: SelectMenuType::Mentionable,
                max_values: Some(5),
                min_values: None,
                options: None,
                placeholder: Some(String::from("Mentionable select menu")),
            })],
        }),
        Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                channel_types: Some(vec![
                    ChannelType::GuildText,
                    ChannelType::PublicThread,
                    ChannelType::PrivateThread,
                    ChannelType::GuildCategory,
                ]),
                custom_id: String::from("channel-select"),
                default_values: if channels.is_empty() {
                    None
                } else {
                    Some(
                        channels
                            .iter()
                            .map(|id| SelectDefaultValue::Channel(*id))
                            .collect(),
                    )
                },
                disabled: false,
                kind: SelectMenuType::Channel,
                max_values: Some(5),
                min_values: None,
                options: None,
                placeholder: Some(String::from("Channel select menu")),
            })],
        }),
    ];

    // Let's hope Discord doesn't add more components without increasing the action-rows-per-message limit Kappa

    if let Err(error) = client
        .create_message(channel)
        .content("Try using the select menus below")
        .components(&components)
        .await
    {
        println!("{error}");
    }
}

async fn handle_select_menu(client: Arc<Client>, interaction: InteractionCreate) {
    let client = client.interaction(*APP_ID.get().unwrap());
    let Some(InteractionData::MessageComponent(data)) = &interaction.data else {
        return;
    };
    if let Err(error) = client
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: None,
                    components: None,
                    custom_id: None,
                    flags: Some(MessageFlags::EPHEMERAL),
                    title: None,
                    allowed_mentions: Some(AllowedMentions::default()),
                    attachments: None,
                    choices: None,
                    embeds: Some(vec![EmbedBuilder::new()
                        .title("Selection information")
                        .color(0x2ccb87)
                        .description(format!(
                            "You interacted with a select menu!\n\
                         - Select menu ID: {}\n\
                         - Selected Values: `{:?}`\n\
                         - Resolved data: `{}`",
                            data.custom_id,
                            data.values,
                            {
                                let mut ret = format!("{:?}", data.resolved);
                                ret.truncate(1000);
                                ret
                            }
                        ))
                        .build()]),
                    tts: Some(false),
                }),
            },
        )
        .await
    {
        println!("{error}");
    }
}

async fn report_error(
    client: InteractionClient<'_>,
    interaction_id: Id<InteractionMarker>,
    token: &str,
) {
    static RESPONSE: OnceLock<InteractionResponse> = OnceLock::new();

    let _ = client
        .create_response(
            interaction_id,
            token,
            RESPONSE.get_or_init(|| InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(
                    InteractionResponseDataBuilder::new()
                        .content("This interaction failed")
                        .flags(MessageFlags::EPHEMERAL)
                        .build(),
                ),
            }),
        )
        .await;
}
