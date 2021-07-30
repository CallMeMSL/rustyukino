#![allow(clippy::needless_lifetimes)]

use std::env;

use serenity::http::client::Http;

use crate::subs_pls::db;
use crate::subs_pls::release_parser::{rss_category_to_show_id, FeedItem};
use crate::subs_pls::release_parser::SubsPlsChannel;
use crate::subs_pls::page_parser::Show;
use serenity::model::id::UserId;

extern crate html_escape;

pub async fn notify_users(feed: &SubsPlsChannel, last_rss: &str) {
    for item in &feed.items {
        if item.guid == last_rss { break; }
        let notification_data = get_notification_data(&*item.category, item).await;
        match notification_data {
            Ok(data) => { send_notifications(data).await }
            Err(e) => {
                let t = match e {
                    NotificationError::DBShowError => "Couldn't fetch show. Probably never added?",
                    NotificationError::DBUsersError => "Error fetching Users.",
                    NotificationError::MappingShowIdError => "Error mapping category to ShowID."
                };
                println!("Error notifying for {}: {}", item.title, t)
            }
        }
    };
}

struct NotificationData<'a> {
    users: Vec<i64>,
    show: Show,
    item: &'a FeedItem,
}

#[derive(Debug)]
pub enum NotificationError {
    MappingShowIdError,
    DBUsersError,
    DBShowError
}

async fn get_notification_data<'a>(show_category: &str, item: &'a FeedItem) -> Result<NotificationData<'a>, NotificationError> {
    let show_id = rss_category_to_show_id(show_category).ok_or_else(|| NotificationError::MappingShowIdError)?;
    let users = db::get_user_ids_for_show_id(&*show_id).await.map_err(|_| NotificationError::DBUsersError)?;
    let show = db::get_show_from_show_id(&*show_id).await.map_err(|_| NotificationError::DBShowError)?;
    Ok(NotificationData { users, show, item })
}

async fn send_notifications<'a>(notification_data: NotificationData<'a>) {
    let http: Http = Http::new_with_token(&env::var("DISCORD_TOKEN").expect("token"));
    for &user_id in notification_data.users.iter() {
        let user_res = UserId::from(user_id as u64).to_user(&http).await;
        match user_res {
            Ok(user) => {
                let d = user.dm(&http, |m| {
                    m.content("");
                    m.embed(|e| {
                        e.title(&notification_data.item.title);
                        e.thumbnail(&notification_data.show.image_url);
                        e.description(&notification_data.show.synopsis);
                        e.field(format!("Download - {}", &notification_data.item.file_size),
                                format!("[🧲](https://yukino.static.app/?r={})", notification_data.item.link), true);
                        e.field("Show Information",
                                format!("[🌐](https://subsplease.org/shows/{}/) [Ⓜ](https://myanimelist.net/search/all?q={}&cat=all)",
                                        notification_data.show.id,
                                        html_escape::encode_text(&notification_data.show.id))
                                , true);

                        e
                    });

                    m
                }).await;
                match d {
                    Ok(_) => {},
                    Err(r) => println!("{}", r.to_string())
                }
            }
            Err(e) => {
                println!("Couldn't find user {} to notify for {}: {}",
                         user_id, &notification_data.show.name, e.to_string())
            }
        }
    }
}