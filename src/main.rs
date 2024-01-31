mod load_dotenv;
use load_dotenv::load_dotenv;

use std::env;

use mc_query::{status, query, rcon};

use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const IP_FINDER: &str = "https://ipinfo.io/ip";

#[poise::command(slash_command)]
async fn sniff(ctx: Context<'_>) -> Result<(), Error> {
    let mc_ip = env::var("SERVERIP").unwrap();
    let mc_port = env::var("SERVERPORT").unwrap().parse::<u16>().unwrap();

    let client = reqwest::Client::new();
    let resp = client.get(IP_FINDER).send().await;
    if resp.is_err() {
        ctx.say("An error occured while getting server ip!").await?;
        return Ok(());
    }
    let resp = resp.unwrap();

    let server_status = status(&mc_ip, mc_port).await;
    if server_status.is_err() {
        ctx.say("An error occured while getting server status!!!").await?;
        return Ok(());
    }
    let server_status = server_status.unwrap();

    let mut em = serenity::CreateEmbed::default()
        .title("Server Info")
        .field("IP:", resp.text().await?, true)
        .field("Version:", &server_status.version.name, true)
        .field("Online / Max:", format!("{} / {}", &server_status.players.online, &server_status.players.max), true);

    if let Some(url) = server_status.favicon {
        em = em.thumbnail(url);
    } else {
        em = em.thumbnail(env::var("MCICONURL").unwrap())
    }

    let reply = poise::CreateReply::default().embed(em);

    ctx.send(reply).await?;

    Ok(())
}


#[poise::command(slash_command)]
async fn command(
    ctx: Context<'_>,
    #[description = "Command you want to run."]
    command: String
) -> Result<(), Error> {
    let mc_ip = env::var("SERVERIP").unwrap();
    let rcon_port = env::var("RCONPORT").unwrap().parse::<u16>().unwrap();
    let rcon_pwd = env::var("RCONPWD").unwrap();
    match rcon::RconClient::new(&mc_ip, rcon_port).await {
        Ok(mut rcon_client) => {
            let auth_result = rcon_client.authenticate(&rcon_pwd).await;
            if auth_result.is_err() {
                ctx.say("Authentication failed!").await?;
                return Ok(());
            }

            match rcon_client.run_command(&command).await {
                Ok(response) => {
                    ctx.say(response).await?;
                },
                Err(e) => {
                    let msg = format!("An error occured while processing command. Error: {e}");
                    ctx.say(msg).await?;
                }
            }
            rcon_client.disconnect().await?;
        },
        Err(e) => {
            let msg = format!("An error occured while connecting RCON. Error: {}", e);
            ctx.say(msg).await?;
        }
    }

    Ok(())
}


#[poise::command(slash_command)]
async fn details(
    ctx: Context<'_>,
    content: DetailsChoice
) -> Result<(), Error> {
    let mc_ip = env::var("SERVERIP").unwrap();
    let query_port = env::var("QUERYPORT").unwrap().parse::<u16>().unwrap();
    match content {
        DetailsChoice::DETAILED => {
            match query::stat_full(&mc_ip, query_port).await {
                Ok(resp) => {
                    let mut em = serenity::CreateEmbed::default()
                        .title("Server Details")
                        .description(&resp.motd)
                        .field("Version:", &resp.version, true)
                        .field("Game ID:", &resp.game_id, true)
                        .field("Game Type:", &resp.game_type, true)
                        .field("Map Name:", &resp.map, true)
                        .field("Online / Max:", format!("{} / {}", &resp.num_players, &resp.max_players), true);
    
                    if !resp.players.is_empty() {
                        let players_string = &resp.players.join("\n* ");
        
                        em = em.field(
                            "Players",
                            format!("* {players_string}"),
                            false
                        )
                    }
        
                    let reply = poise::CreateReply::default().embed(em);
        
                    ctx.send(reply).await?;
                },
                Err(e) => {
                    let msg = format!("An error occured while querying full stats! Error: {e}");
                    ctx.say(msg).await?;
                }
            }

        },
        DetailsChoice::SIMPLE => {
            match query::stat_basic(&mc_ip, query_port).await {
                Ok(resp) => {
                    let em = serenity::CreateEmbed::default()
                        .title("Server Details")
                        .description(&resp.motd)
                        .field("Game Type:", &resp.game_type, true)
                        .field("Map Name:", &resp.map, true)
                        .field("Online / Max:", format!("{} / {}", &resp.num_players, &resp.max_players), true);

                    let reply = poise::CreateReply::default().embed(em);

                    ctx.send(reply).await?;
                },
                Err(e) => {
                    let msg = format!("An error occured while querying basic stats! Error: {e}");
                    ctx.say(msg).await?;
                }
            }

            
        }
    }

    Ok(())
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error>{
    load_dotenv(None).unwrap();
    let tkn = env::var("TOKEN").unwrap();

    let intents = serenity::GatewayIntents::all();

    let framework: poise::Framework<Data, Error> = poise::Framework::builder()
        .options(poise::FrameworkOptions { 
            commands: vec![sniff(), command(), details()],
            skip_checks_for_owners: true,
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
         })
         .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
         }).build();

    let mut bot = serenity::ClientBuilder::new(tkn, intents).framework(framework).await?;
    bot.start().await?;

    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot: _ } = event {
        println!("Bot is ready!!!");
        ctx.set_activity(Some(serenity::ActivityData::custom("Like the pickle in my burger, i wanna take you out.")));
    }

    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum DetailsChoice {
    #[name = "Simple"]
    SIMPLE,
    #[name = "Detailed"]
    DETAILED
}