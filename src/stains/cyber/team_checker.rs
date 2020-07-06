use crate::{
  collections::team::players,
  stains::cyber::{
    types::*,
    utils::{ get_race2, get_map }
  }
};

use serenity::{
  prelude::*
};

use reqwest;

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
  pub static ref GAMES: Mutex<HashMap<String, TrackingGame>> = Mutex::new(HashMap::new());
}

pub fn check_match(matchid_lol : &str) -> Option<(String, u32, Option<(String, String, String, String)>)> {

  let mut matchid_s : String = String::new();
  if let Ok(wtf) = reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches?offset=0&gateway=20") {
    if let Ok(going) = wtf.json::<Going>() {
      for mm in &going.matches {
        if mm.startTime == matchid_lol {
          // TODO: change that hack one day
          if players().into_iter().any(|playa|
            if mm.gameMode == 6 || mm.gameMode == 2 {
              mm.teams[0].players[0].battleTag == playa.battletag || mm.teams[1].players[0].battleTag == playa.battletag ||
              mm.teams[0].players[1].battleTag == playa.battletag || mm.teams[1].players[1].battleTag == playa.battletag
            } else {
              mm.teams[0].players[0].battleTag == playa.battletag || mm.teams[1].players[0].battleTag == playa.battletag
            }
          ) {
            matchid_s = mm.id.clone();
            break;
          }
        }
      }
    }
  }

  if matchid_s.is_empty() { return None; }
  let matchid = matchid_s.as_str();
  let url = format!("https://statistic-service.w3champions.com/api/matches/{}", matchid);

  if let Ok(res) = reqwest::blocking::get(url.as_str()) {
    match res.json::<MD>() {
      Ok(md) => {
        let m = md.match_data;
        let mstr_o =
        if m.gameMode == 1 {
          set!{ g_map = get_map(m.map.as_str())
              , race1 = get_race2(m.teams[0].players[0].race)
              , race2 = get_race2(m.teams[1].players[0].race) };
          let player1 = if m.teams[0].players[0].won {
            format!("__**{}**__ **+{}**", m.teams[0].players[0].name, m.teams[0].players[0].mmrGain)
          } else {
            format!("__*{}*__ **{}**", m.teams[0].players[0].name, m.teams[1].players[0].mmrGain)
          };
          let player2 = if m.teams[1].players[0].won {
            format!("__**{}**__ **+{}**", m.teams[1].players[0].name, m.teams[0].players[0].mmrGain)
          } else {
            format!("__*{}*__ **{}**", m.teams[1].players[0].name, m.teams[1].players[0].mmrGain)
          };
          Some(format!("({}) {} [{}] *vs* ({}) {} [{}] *{}*",
              race1, player1, m.teams[0].players[0].oldMmr
            , race2, player2, m.teams[1].players[0].oldMmr, g_map))
        } else if m.gameMode == 6 || m.gameMode == 2 {
          set!{ g_map = get_map(m.map.as_str())
              , race1 = get_race2(m.teams[0].players[0].race)
              , race12 = get_race2(m.teams[0].players[1].race)
              , race2 = get_race2(m.teams[1].players[0].race)
              , race22 = get_race2(m.teams[1].players[1].race) };
          if m.gameMode == 6 {
            if m.teams[0].won {
              Some(format!("({}+{}) __**{} + {} [{}]**__ **+{}** (won)\n*vs*\n({}+{}) __*{} + {} [{}]*__ *{}* (lost)\n\nmap: **{}**",
                race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, g_map))
            } else {
              Some(format!("({}+{}) __*{} + {} [{}]*__ *{}* (lost)\n*vs*\n({}+{}) __**{} + {} [{}]**__ **+{}** (won)\n\nmap: **{}**",
                race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, g_map))
            }
          } else {
            if m.teams[0].won {
              Some(format!("({}+{}) __**{} [{}]**__ **+{}** + __**{} [{}]**__ **+{}** (won)\n*vs*\n({}+{}) __*{} [{}]*__ *{}* + __*{} [{}]*__ *{}* (lost)\n\nmap: **{}**",
                race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr, m.teams[0].players[1].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, m.teams[1].players[1].mmrGain, g_map))
            } else {
              Some(format!("({}+{}) __*{} [{}]*__ *{}* + __*{} [{}]*__ *{}* (lost)\n*vs*\n({}+{}) __**{} [{}]**__ **+{}** + __**{} [{}]**__ **+{}** (won)\n\nmap: **{}**",
              race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr, m.teams[0].players[1].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, m.teams[1].players[1].mmrGain, g_map))
            }
          }
        } else {
          None
        };
        match mstr_o {
          Some(mstr) => {
            let duration_in_minutes = m.durationInSeconds / 60;
            if md.playerScores.len() > 1 && m.gameMode == 1 {
              set! { p1 = &md.playerScores[0]
                   , p2 = &md.playerScores[1]
                   , s1 = p1.battleTag.clone()
                   , s2 = p2.battleTag.clone() };
              let s3 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
                  , p1.unitScore.unitsProduced
                  , p1.unitScore.unitsKilled
                  , p1.resourceScore.goldCollected
                  , p1.heroScore.expGained);
              let s4 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
                  , p2.unitScore.unitsProduced
                  , p2.unitScore.unitsKilled
                  , p2.resourceScore.goldCollected
                  , p2.heroScore.expGained);
              let scores = if m.teams[0].players[0].battleTag == s1 {
                  Some((s1,s2,s3,s4))
                } else {
                  Some((s2,s1,s4,s3))
                };
              return Some((mstr, duration_in_minutes, scores));
            }
            return Some((mstr, duration_in_minutes, None));
          },
          None => {
            return None;
          }
        }
      },
      Err(err) => {
        error!("Failed parse MD {:?}", err);
      }
    }
  }
  None
}

pub fn check(ctx : &Context, channel_id : u64) -> Vec<StartingGame> {
  let mut out : Vec<StartingGame> = Vec::new();
  if let Ok(res) =
    // getaway 20 = Europe
    reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20") {
    if let Ok(going) = res.json::<Going>() {
      if going.matches.len() > 0 {
        if let Ok(mut games_lock) = GAMES.lock() {

          for m in going.matches {
            if m.gameMode == 1 {
              if m.teams.len() > 1 && m.teams[0].players.len() > 0 && m.teams[1].players.len() > 0 {
                if let Some(playa) = players().into_iter().find(|p|
                     m.teams[0].players[0].battleTag == p.battletag
                  || m.teams[1].players[0].battleTag == p.battletag
                ) {
                  let g_map = get_map(m.map.as_str());
                  let race1 = get_race2(m.teams[0].players[0].race);
                  let race2 = get_race2(m.teams[1].players[0].race);
                  let mstr = format!("({}) **{}** [{}] *vs* ({}) **{}** [{}] *{}*",
                    race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                  , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);

                  if let Some(track) = games_lock.get_mut(m.startTime.as_str()) {
                    track.still_live = true;
                    let minutes = track.passed_time / 2;
                    let footer = format!("Passed: {} min", minutes);

                    if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id) {
                      if let Ok(user) = ctx.http.get_user(playa.discord) {

                        let mut fields = Vec::new();
                        let mut img = None;
                        let mut url = None;
                        if msg.embeds.len() > 0 && msg.embeds[0].fields.len() > 0 {
                          for f in msg.embeds[0].fields.clone() {
                            fields.push((f.name, f.value, f.inline));
                          }
                          img = msg.embeds[0].image.clone();
                          url = msg.embeds[0].url.clone();
                        };

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e|  {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&user.name))
                              .description(mstr)
                              .footer(|f| f.text(footer));
                            if fields.len() > 0 {
                              e = e.fields(fields);
                            }
                            if img.is_some() {
                              e = e.image(img.unwrap().url);
                            }
                            if url.is_some() {
                              e = e.url(url.unwrap());
                            }
                            e
                          }
                        )) {
                          error!("Failed to post live match {:?}", why);
                        }
                      }
                    }

                  } else {
                    out.push(
                      StartingGame {
                        key: m.startTime,
                        description: mstr,
                        user: playa.discord,
                        stream: playa.streams
                      }
                    );
                  }
                }
              }
            } else if m.gameMode == 6 || m.gameMode == 2 { // AT or RT mode
              if m.teams.len() > 1 && m.teams[0].players.len() > 1 && m.teams[1].players.len() > 1 {
                if let Some(playa) = players().into_iter().find(|p|
                     m.teams[0].players[0].battleTag == p.battletag
                  || m.teams[1].players[0].battleTag == p.battletag
                  || m.teams[0].players[1].battleTag == p.battletag
                  || m.teams[1].players[1].battleTag == p.battletag) {

                  let g_map = get_map(m.map.as_str());

                  set! { race1 = get_race2(m.teams[0].players[0].race)
                       , race12 = get_race2(m.teams[0].players[1].race)
                       , race2 = get_race2(m.teams[1].players[0].race)
                       , race22 = get_race2(m.teams[1].players[1].race) };

                  let mstr = if m.gameMode == 6 {
                    format!("({}+{}) **{}** + **{}** [{}]\n*vs*\n({}+{}) **{}** + **{}** [{}]\n\nmap: **{}**",
                      race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr
                    , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, g_map)
                  } else {
                    format!("({}+{}) **{}** [{}] + **{}** [{}]\n*vs*\n({}+{}) **{}** [{}] + **{}** [{}]\n\nmap: **{}**",
                      race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr
                    , race2, race22, m.teams[1].players[0].name, m.teams[0].players[0].oldMmr, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, g_map)
                  };

                  if let Some(track) = games_lock.get_mut(m.startTime.as_str()) {
                    track.still_live = true;
                    let minutes = track.passed_time / 2;
                    let footer = format!("Passed: {} min", minutes);
                    if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id) {
                      if let Ok(user) = ctx.http.get_user(playa.discord) {
                        let mut fields = Vec::new();
                        let mut img = None;
                        let mut url = None;
                        if msg.embeds.len() > 0 && msg.embeds[0].fields.len() > 0 {
                          for f in msg.embeds[0].fields.clone() {
                            fields.push((f.name, f.value, f.inline));
                          }
                          img = msg.embeds[0].image.clone();
                          url = msg.embeds[0].url.clone();
                        };

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e| {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&user.name))
                              .description(mstr)
                              .footer(|f| f.text(footer));
                            if fields.len() > 0 {
                              e = e.fields(fields);
                            }
                            if img.is_some() {
                              e = e.image(img.unwrap().url);
                            }
                            if url.is_some() {
                              e = e.url(url.unwrap());
                            }
                            e
                          }
                        )) {
                          error!("Failed to post live match {:?}", why);
                        }
                      }
                    }
                  } else {
                    out.push(
                      StartingGame {
                        key: m.startTime,
                        description: mstr,
                        user: playa.discord,
                        stream: playa.streams
                      }
                    );
                  }

                }
              }
            }
          }

          let mut k_to_del : Vec<String> = Vec::new();
          for (k, track) in games_lock.iter_mut() {
            if !track.still_live {
              if let Some((new_text, duration, fields)) = check_match(k) {
                if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id) {
                  let footer : String = format!("Passed: {} min", duration);
                  if let Ok(user) = ctx.http.get_user(track.tracking_usr_id) {
                    match fields {
                      Some((s1,s2,s3,s4)) => {
                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e| e
                          .author(|a| a.icon_url(&user.face()).name(&user.name))
                          .description(new_text)
                          .fields(vec![
                            (s1, s3, true),
                            (s2, s4, true)
                          ])
                          .footer(|f| f.text(footer))
                        )) {
                            error!("Failed to update live match {:?}", why);
                        }
                      }, None => {
                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e| e
                          .author(|a| a.icon_url(&user.face()).name(&user.name))
                          .description(new_text)
                          .footer(|f| f.text(footer))
                        )) {
                            error!("Failed to update live match {:?}", why);
                        }
                      }
                    };
                  }
                }
                // we only delete match if it's passed
                // if not possibly there is a bug and we're waiting for end
                k_to_del.push(k.clone());
              }
            }
          }
          for ktd in k_to_del {
            games_lock.remove(ktd.as_str());
          }

        }
      }
    }
  }
  out
}
