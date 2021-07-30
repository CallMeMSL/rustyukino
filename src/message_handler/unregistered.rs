use serenity::client::Context;
use serenity::model::channel::Message;
use super::split_at_fist_space;
use crate::user_manager;

pub async fn main(ctx: Context, msg: Message) {
    let (op, arg) = split_at_fist_space(&msg.content).await;
    match (op.as_str(), arg.as_str()) {
        ("register", _) => { register(ctx, msg).await; }
        ("help", _) => { help(ctx, msg).await; }
        _ => { msg.reply(ctx, "Command not recognized. Use the help command for a list of actions.").await.ok(); }
    };
}

async fn register(ctx: Context, msg: Message) {
    let register_res = user_manager::register_user(msg.author.id.0 as i64).await;
    let reply = match register_res {
        Ok(_) => "Successfully registered!",
        Err(_) => "An error has occurred while registering. Please try again later."
    };
    msg.reply(ctx, reply).await.ok();
}

async fn help(ctx: Context, msg: Message) {
    let titles = ["register", "help"];
    let descriptions = ["Type this to unlock the functionality of the bot. Your UserID will be saved.",
        "Shows this message"];
    msg.channel_id.send_message(ctx, |m| {
        m.content("");
        m.embed(|e| {
            for (t, d) in titles.iter().zip(&descriptions) {
                e.field(t, d, false);
            }
            e
        });
        m
    }).await.ok();
}