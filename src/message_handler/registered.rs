use serenity::client::Context;
use serenity::model::channel::Message;

use crate::user_manager;

use super::split_at_fist_space;
use crate::subs_pls::page_parser::AddFailure;
use crate::user_manager::RemoveFailure;


pub async fn main(ctx: Context, msg: Message) {
    let (op, arg) = split_at_fist_space(&msg.content).await;
    match (op.as_str(), arg.as_str()) {
        ("help", _) => { help(ctx, msg).await }
        ("unregister", _) => { unregister(ctx, msg).await }
        ("add", ident) => { add(ctx, msg, ident).await }
        ("remove", "non-airing") => { remove_na(ctx, msg).await }
        ("remove", ident) => { remove(ctx, msg, ident).await }
        ("schedule", "") => { schedule(ctx, msg).await }
        ("examples", _) => { examples(ctx,msg).await}
        _ => { msg.channel_id.say(ctx, "Command not recognized. Use the help command for a list of actions.").await.ok(); }
    };
}


async fn help(ctx: Context, msg: Message) {
    let titles = ["help", "unregister", "add", "remove", "schedule", "examples"];
    let descriptions = ["Shows this message",
        "This will remove everything about you & your saved shows from the database.",
        "With add you can extend your watchlist. Pass with the argument a valid link of the shows overview page.",
        "Remove lets you scrap shows from your watchlist. You can either use a link, the exact show name or the \"non-airing\"
         keyword to remove all non airing-shows.",
        "Prints a personal release schedule.",
        "Couple of examples on how to use this bot."
        ];
    let s = msg.channel_id.send_message(ctx, |m| {
        m.content("");
        m.embed(|e| {
            for (t, d) in titles.iter().zip(&descriptions) {
                e.field(t, d, false);
            }
            e
        });
        m
    }).await;
    if s.is_err() {
        println!("Discord Error: {}", { s.err().unwrap().to_string() });
    }
}

async fn unregister(ctx: Context, msg: Message) {
    let register_res = user_manager::unregister_user(msg.author.id.0 as i64).await;
    let reply = match register_res {
        Ok(_) => "Successfully unregistered! Good bye!",
        Err(_) => "An error has occurred while unregistering. Please try again later."
    };
    msg.reply(ctx, reply).await.ok();
}

async fn add(ctx: Context, msg: Message, identifier: &str) {
    let res = user_manager::add_user_show(msg.author.id.0 as i64, identifier).await;

    match res {
        Ok(show) => {
            msg.channel_id.send_message(ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title("Show successfully added!");
                    e.field(&show.name, &show.synopsis, false);
                    e.image(&show.image_url);
                    if show.air_time.is_airing {
                        e.field("Is airing currently. Estimated release: ", &show.air_time.to_string(), true);
                    } else {
                        e.field("Currently not airing.", "Check the Website for further information.", true);
                    }
                    e
                });
                m
            }).await
        }
        Err(AddFailure::AlreadyAdded) => msg.reply(ctx, "show already added.").await,
        Err(AddFailure::InvalidUrl) => msg.reply(ctx, "Invalid url. Use the url of a show page.").await,
        Err(AddFailure::ShowNotAvailable) => msg.reply(ctx, "This show doesn't exist. Please check the identifier in the url.").await,
        Err(AddFailure::DatabaseError) => msg.reply(ctx, "Error communicating with database. Try again later.").await,
        Err(AddFailure::NameNotFound) => msg.reply(ctx, "Adding by name is not supported (yet).").await
    }.ok();
}

async fn remove_na(ctx: Context, msg: Message) {
    let removed_shows_res = user_manager::remove_non_airing(msg.author.id.0 as i64).await;
    match removed_shows_res {
        Ok(shows) if !shows.is_empty() => {
            msg.channel_id.send_message(ctx, |m| {
                m.content("");
                m.embed(|e| {
                    e.title("The following shows have been removed from your watchlist:");
                    for show in shows.iter() {
                        e.field(&show.name, &show.synopsis, false);
                    }
                    e
                });
                m
            }).await
        }
        Ok(_) => msg.reply(ctx, "I haven't found any shows on your watchlist, that aren't airing.").await,
        Err(_) => msg.reply(ctx, "Something went wrong and only some or no shows at \
        all have been removed. Try again later or remove the rest manually.").await
    }.ok();
}

async fn remove(ctx: Context, msg: Message, identifier: &str) {
    let res = user_manager::remove_user_show(msg.author.id.0 as i64, identifier).await;
    match res {
        Ok(()) => msg.reply(ctx, "Show from watchlist removed.").await.ok(),
        Err(RemoveFailure::InvalidIdentifier) => msg.reply(ctx, "Invalid url.").await.ok(),
        Err(RemoveFailure::DBError) => msg.reply(ctx, "Error communicating with database. Try again later.").await.ok(),
        Err(RemoveFailure::ShowNotFound) => msg.reply(ctx, "I couldn't find a matching show in your watchlist.\
            Give me a _correct_ the url of the show with this command.").await.ok()
    };
}

async fn schedule(ctx: Context, msg: Message) {
    let table_res = user_manager::generate_schedule(msg.author.id.0 as i64).await;
    match table_res {
        Ok(table_res) => {
            let data = table_res.get_printable_table();
            match data {
                Ok(d) => {
                    msg.channel_id.send_message(ctx, |m| {
                        m.content("");
                        m.embed(|e| {
                            e.title("Currently Watching:");
                            for (day, shows) in d {
                                if shows != "".to_string() { e.field(day, shows, false); }
                            };
                            e
                        });
                        m
                    }).await
                }
                Err(_) => msg.reply(ctx, "Error Generating Table.").await
            }
        }
        Err(_) => msg.reply(ctx, "Something went wrong, try again later.").await
    }.ok();
}

async fn examples(ctx: Context, msg: Message) {
    msg.channel_id.say(ctx,
        "
        ```haskell
        -- add show
        add https://subsplease.org/shows/one-piece/
        -- remove show
        remove https://subsplease.org/shows/one-piece/
        -- also possible
        remove One Piece
        -- delete everything about me
        unregister
        -- display schedule
        schedule```
        "
    ).await.ok();
}