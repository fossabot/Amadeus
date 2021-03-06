use crate::{
  common::{
    types::AOptions,
    msg::{ direct_message, reply },
    conf
  }
};

use serenity::{
  model::{ misc::Mentionable
         , id::GuildId, id::ChannelId
         , channel::* },
  client::{ bridge::voice::ClientVoiceManager },
  voice,
  prelude::*,
  framework::standard::{
    Args, Delimiter, CommandResult,
    macros::command
  }
};

use std::sync::Arc;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub async fn rejoin_voice_channel(ctx : &Context, conf: &AOptions) {
  if conf.rejoin {
    set!{ last_guild_u64 = conf.last_guild.parse::<u64>().unwrap_or(0)
        , last_channel_u64 = conf.last_channel.parse::<u64>().unwrap_or(0) };
    if last_guild_u64 != 0 && last_channel_u64 != 0 {
      set!{ last_guild_conf = GuildId( last_guild_u64 )
          , last_channel_conf = ChannelId( last_channel_u64 ) };
      let manager_lock =
        ctx.data.read().await
          .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
      let mut manager = manager_lock.lock().await;
      if manager.join(last_guild_conf, last_channel_conf).is_some() {
        info!("Rejoined voice channel: {}", last_channel_conf);
        if conf.last_stream != "" {
          if let Some(handler) = manager.get_mut(last_guild_conf) {
            let source = match voice::ytdl(&conf.last_stream).await {
              Ok(source) => source,
              Err(why) => {
                error!("Err starting source: {:?}", why);
                return ();
              }
            };
            handler.play(source);
          }
        }
      } else {
        error!("Failed to rejoin voice channel: {}", last_channel_conf);
      }
    }
  }
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
  let guild = match msg.guild(&ctx).await {
    Some(guild) => guild,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    }
  };
  let guild_id = guild.id;
  let channel_id = guild
    .voice_states.get(&msg.author.id)
    .and_then(|voice_state| voice_state.channel_id);
  let connect_to = match channel_id {
    Some(channel) => channel,
    None => {
      reply(ctx, msg, "You're not in a voice channel").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
    .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if manager.join(guild_id, connect_to).is_some() {
    let mut conf = conf::parse_config();
    let last_guild_conf = GuildId( conf.last_guild.parse::<u64>().unwrap_or(0) );
    let last_channel_conf = ChannelId( conf.last_channel.parse::<u64>().unwrap_or(0) );
    if last_guild_conf != guild_id || last_channel_conf != connect_to || conf.rejoin == false {
      conf.rejoin = true;
      conf.last_guild = format!("{}", guild_id);
      conf.last_channel = format!("{}", connect_to);
      conf::write_config(&conf);
    }
    if let Err(why) = msg.channel_id.say(&ctx, &format!("I've joined {}", connect_to.mention())).await {
      error!("failed to say joined {:?}", why);
    }
  } else {
    direct_message(ctx, msg, "Some error joining the channel...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    },
  };
  let manager_lock = ctx.data.read()
      .await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  let has_handler = manager.get(guild_id).is_some();
  if has_handler {
    manager.remove(guild_id);
    let _ = msg.channel_id.say(&ctx, "I left voice channel");
    let mut conf = conf::parse_config();
    if conf.rejoin {
      conf.rejoin = false;
      conf::write_config(&conf);
    }
  } else {
    reply(ctx, &msg, "I'm not in a voice channel").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let url =
    if args.len() > 0 {
      match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
          reply(ctx, msg, "You must provide a URL to a video or audio").await;
          return Ok(());
        }
      }
    } else {
      let conf = conf::parse_config();
      conf.last_stream
    };
  if !url.starts_with("http") {
    reply(ctx, msg, "You must provide a valid URL").await;
    return Ok(());
  }
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      reply(ctx, msg, "Error finding channel info...").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
      .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if let Some(handler) = manager.get_mut(guild_id) {
    let source = match voice::ytdl(&url).await {
      Ok(source) => source,
      Err(why) => {
        error!("Err starting source: {:?}", why);
        reply(ctx, msg, &format!("Sorry, error sourcing ffmpeg {:?}", why)).await;
        return Ok(());
      }
    };
    handler.play_only(source);
    let mut conf = conf::parse_config();
    let last_stream_conf = conf.last_stream;
    if last_stream_conf != url {
      conf.last_stream = url.clone();
      conf::write_config(&conf);
    }
    reply(ctx, msg, &format!("playing stream: {}", url)).await;
  } else {
    reply(ctx, msg, "Not in a voice channel to play in...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
  play(ctx, msg, Args::new("", &[Delimiter::Single(' ')])).await
}
