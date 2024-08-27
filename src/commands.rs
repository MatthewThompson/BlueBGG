use arnak::{Collection, CollectionItemBrief, CollectionItemType, CollectionQueryParams};
use poise::serenity_prelude::{
    Colour, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, Timestamp,
};
use poise::CreateReply;

use crate::{Context, Error};

/// Show this help menu
#[poise::command(track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    println!("help command");
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Test help text idk...",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

// Creates an embed with the default fields set.
//
// This sets the title to the bot name, and adds the footer.
fn create_base_embed() -> CreateEmbed {
    CreateEmbed::new()
        .footer(CreateEmbedFooter::new(
            "Bot source: https://github.com/MatthewThompson/BlueBGG",
        ))
        .colour(Colour::from_rgb(0, 176, 255))
        .timestamp(Timestamp::now())
}

/// Get info for a particular game, by its ID.
#[poise::command(track_edits, slash_command)]
pub async fn game_info(
    ctx: Context<'_>,
    #[description = "An ID of a game in the BGG database."] game_id: u64,
) -> Result<(), Error> {
    let reply_content = format!("Getting game {}...Not yet implemented", game_id);
    let embed = create_base_embed()
        .author(CreateEmbedAuthor::new("Some game"))
        .description(&reply_content);
    let reply = CreateReply::default().embed(embed);

    ctx.send(reply).await?;
    Ok(())
}

/// Get the 10 games that a given user has rated highest.
#[poise::command(track_edits, slash_command)]
pub async fn top10(
    ctx: Context<'_>,
    #[description = "A board game geek's user to get the top 10 rated games for."]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    bgg_user: String,
) -> Result<(), Error> {
    let params = CollectionQueryParams::new()
        .item_type(CollectionItemType::BoardGame)
        .exclude_item_type(CollectionItemType::BoardGameExpansion);

    let user_collection = ctx
        .data()
        .board_game_api
        .collection_brief()
        .get_from_query(&bgg_user, params)
        .await;
    let games = match user_collection {
        Err(arnak::Error::UnknownUsernameError) => {
            ctx.reply(format!("User {} not found", bgg_user)).await?;
            return Ok(());
        },
        Err(arnak::Error::MaxRetryError(_)) => {
            ctx.reply(format!(
                "Data for {} from BGG not yet ready, please try again shortly",
                bgg_user
            ))
            .await?;
            return Ok(());
        },
        Err(e) => {
            println!("{:?}", e);
            ctx.reply(format!(
                "Unexpected error requesting user {} collection",
                bgg_user
            ))
            .await?;
            return Ok(());
        },
        Ok(user_collection) => get_games_by_user_rating_desc(user_collection),
    };

    let reply_content = games
        .iter()
        .take(10)
        .map(|game| {
            let user_rating = match game.stats.rating.user_rating {
                None => String::from("Unrated"),
                Some(rating) => rating.to_string(),
            };
            format!(
                "- [{}](https://boardgamegeek.com/boardgame/{}) - ({})\n",
                game.name, game.id, user_rating
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let embed = create_base_embed()
        .author(
            CreateEmbedAuthor::new(format!("{}'s top 10", bgg_user)).url(format!(
                "https://boardgamegeek.com/collection/user/{}",
                bgg_user
            )),
        )
        .description(&reply_content);
    let reply = CreateReply::default().embed(embed);

    ctx.send(reply).await?;
    Ok(())
}

fn get_games_by_user_rating_desc(
    user_collection: Collection<CollectionItemBrief>,
) -> Vec<CollectionItemBrief> {
    let mut games = user_collection.items;
    games.sort_unstable_by(|b, a| {
        a.stats
            .rating
            .user_rating
            .partial_cmp(&b.stats.rating.user_rating)
            .unwrap()
    });
    games
}
