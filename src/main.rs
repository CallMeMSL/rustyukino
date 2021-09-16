use std::env;
use std::error::Error;


use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use tokio::spawn;
use tokio_schedule::{every, Job};

use crate::subs_pls::db::RssIdDbCommunicator;
use crate::subs_pls::notify::notify_users;
use crate::subs_pls::release_parser::SubsPlsChannel;
use crate::user_manager::is_user_registered;

mod subs_pls;
mod user_manager;
mod message_handler;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() && !msg.author.bot { message_handler(ctx, msg).await; }
    }
}

async fn message_handler(ctx: Context, msg: Message) {
    msg.channel_id.broadcast_typing(&ctx).await.ok();
    let is_registered_res = is_user_registered(msg.author.id.0 as i64).await;
    match is_registered_res {
        Ok(true) => {message_handler::registered::main(ctx, msg).await;}
        Ok(false) => {message_handler::unregistered::main(ctx, msg).await;}
        Err(_) => { msg.channel_id.say(ctx, "Error communicating with database. Try again later.").await.ok(); }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.no_dm_prefix(true));
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");


    let release_check = every((&*env::var("RSS_REFRESH")
        .expect("rss refresh")).parse()?)
        .second().perform(|| async {
        check_rss(env::var("RSS_LINK").expect("rss link")).await;
    });
    spawn(release_check);

    let eu = every(1).day().perform(|| async {
        println!("Updating shows");
        episode_update().await
    });
    spawn(eu);


    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }


    Ok(())
}

async fn check_rss(rss_link: String) {
    let rss_db_communicator: RssIdDbCommunicator = RssIdDbCommunicator::new().await;
    let rss =
        match reqwest::get(&rss_link).await {
            Ok(r) => match r.text().await {
                Ok(t) => t,
                _ => "not available".to_string()
            }
            _ => "not available".to_string()
        };
    let feed_res = SubsPlsChannel::from_xml(&rss);
    match feed_res {
        Ok(feed) => {
            let last_rss = rss_db_communicator.get_guid().await;
            let new_newest = feed.items[0].guid.to_string();
            if new_newest != last_rss {
                notify_users(&feed, &last_rss).await;
                rss_db_communicator.save_guid(&new_newest).await.ok();
            }
        }
        Err(e) => {println!("Rss parsing Error: {}", e)}
    }

}

async fn episode_update() {
    subs_pls::update_shows::update_shows().await
}



