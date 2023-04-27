use mysql_async::Conn;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::prelude::Context;
use serenity::prelude::SerenityError;
use serenity::utils::Colour;

use crate::engine::{parse_set};
use crate::sentences::generate_sentences;
use crate::shared::console_stamp;

pub async fn run(interaction: &ApplicationCommandInteraction, ctx: &Context, db: Conn) -> Result<(), SerenityError> {

    let raw_set = match interaction.data.options.get(0).expect("").resolved.as_ref().expect("") {
        CommandDataOptionValue::String(value) => value,
        _ => return Ok(())
    };
    let set = match parse_set(raw_set) {
        Ok(set) => set,
        Err(error) => {
            return interaction.create_interaction_response(&ctx.http, 
                |r| r.interaction_response_data(|m|
                    m
                    .ephemeral(true)
                    .embed(|e|
                        e.description(error).colour(Colour::from_rgb(255, 0, 0))
                    )
                )
            ).await;
        }
    };

    match generate_sentences(vec![], vec![set], db, "", "").await {
        Ok(result) => interaction.create_interaction_response(&ctx.http, |r| r.interaction_response_data(
            |m|
            m.content(result)
        )).await,
        Err(error) => {
            println!("{}Error: {}", console_stamp(), error);
            interaction.create_interaction_response(&ctx.http, 
                |r| r.interaction_response_data(|m|
                    m
                    .ephemeral(true)
                    .embed(|e|
                        e.description(error).colour(Colour::from_rgb(255, 0, 0))
                    )
                )
            ).await
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("try-set").description("Test out a pronoun set!")
    .create_option(|option| {
        option
            .name("set")
            .description("A pronoun set to try!")
            .kind(CommandOptionType::String)
            .required(true)
    })
    .create_option(|option| {
        option
            .name("name")
            .description("A name to try!")
            .kind(CommandOptionType::String)
            .required(false)
    })
    .create_option(|option| {
        option
            .name("hidden")
            .description("Whether or not others can see the output")
            .kind(CommandOptionType::Boolean)
            .required(false)
    })
}
