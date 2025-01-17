#![feature(str_split_once)]

use std::{env, time::Instant};

use rand::Rng;
use serde::Deserialize;
use serenity::{
    async_trait,
    http::typing::Typing,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const LISTA: &str = "https://docs.google.com/spreadsheets/d/1_2EKicfuSAUhUHD4V6ey_nhgAqrF_GBlDRLXdYjvvfw/gviz/tq?tqx=out:csv&sheet=introducción&range=D47:R47";
const CODEFORCES: &str = "https://codeforces.com";
const OMEGAUP_RNDM: &str = "https://omegaup.com/problem/random/language/";
const OMEGAUP: &str = "https://omegaup.com";
const UVA: &str = "https://onlinejudge.org/index.php?option=com_onlinejudge&Itemid=8&category=24&page=show_problem&problem=";

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("No hay token");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Error al crear cliente");

    if let Err(why) = client.start().await {
        eprintln!("Error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let now = Instant::now();

        let respuesta = match msg.content.trim() {
            "!cf" => {
                let mut rng = rand::thread_rng();

                Some(format!(
                    "{}/problemset/problem/{}/{}",
                    CODEFORCES,
                    rng.gen_range(1..=1452).to_string(),
                    rng.gen_range('A'..='H')
                ))
                //TODO es necesario revisar url? Codeforces siempre te redirecciona
            }

            "!oup" => {
                let response = reqwest::get(OMEGAUP_RNDM).await.unwrap();
                Some(format!("{}{}", OMEGAUP, response.url().path()))
            }

            "!uva" => None, //TODO problema random de UVA

            "!top" => {
                let medallas = ["🥇", "🥈", "🥉"];

                //ver https://stackoverflow.com/questions/33713084/download-link-for-google-spreadsheets-csv-export-with-multiple-sheets
                let datos = reqwest::get(LISTA)
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
                    .replace("\"", " ");

                let mut podium: Vec<(&str, u8)> = datos
                    .split(',')
                    .filter_map(|persona| persona.split_once(':'))
                    .map(|(nombre, problema)| (nombre, problema.trim().parse().unwrap_or(0)))
                    .collect();

                //ordena respecto al número de problemas, en orden descendiente.
                podium.sort_unstable_by(|p, q| q.1.cmp(&p.1));

                let mut text = String::from("Problemas hechos\n");

                for ((nombre, problemas), medalla) in podium.iter().take(3).zip(medallas.iter()) {
                    text.push_str(&format!("{} {} {}\n", medalla, nombre, problemas));
                }

                Some(text)
            }

            _ => {
                if msg.content.contains("!uva") {
                    //Hay un offset entre el número del problema y el número del url
                    //Ej. El problema 100 es el 36 en el url, por eso la resta.
                    let problme_num: u32 = msg.content[5..].trim_end().parse::<u32>().unwrap() - 64;
                    Some(format!("{}{}", UVA, problme_num))
                } else if msg.content.contains("!cf") {
                    //TODO
                    // Problemas en rango de dificultad
                    //https://codeforces.com/problemset?tags=100-800

                    let typing = Typing::start(ctx.http.clone(), *msg.channel_id.as_u64()).unwrap();
                    let categorias = msg.content[3..]
                        .split_whitespace()
                        .fold(String::from("?tags="), |acc, x| format!("{}{};", acc, x));

                    let response = reqwest::get(&format!(
                        "{}/api/problemset.problems{}",
                        CODEFORCES, categorias
                    ))
                    .await
                    .unwrap()
                    .json::<Root>()
                    .await
                    .unwrap()
                    .result;

                    let mut rng = rand::thread_rng();

                    let mut respuesta = format!("Categoría: {}\n\n", &msg.content[3..]);

                    for _ in 0..3 {
                        let idx = rng.gen_range(0..response.problems.len());

                        let problem = &response.problems[idx];

                        let link = format!(
                            "{}/problemset/problem/{}/{}",
                            CODEFORCES, problem.contest_id, problem.index,
                        );

                        respuesta.push_str(&format!(
                            "{}\nDificultad: {}\n{}\n\n",
                            problem.name,
                            problem.rating.unwrap_or(0),
                            link
                        ));
                    }

                    typing.stop();
                    Some(respuesta)
                } else {
                    None
                }
            }
        };

        if let Some(respuesta) = respuesta {
            //TODO logs
            println!(
                "{} {} {:#?}",
                msg.author.name,
                msg.content,
                Instant::now().duration_since(now)
            );

            if let Err(e) = msg.channel_id.say(&ctx.http, respuesta).await {
                eprintln!("Error al mandar mensaje: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, _: Ready) {
        println!("Listo para responder.");
    }
}

#[derive(Deserialize)]
pub struct Root {
    result: Result,
}

#[derive(Deserialize)]
pub struct Result {
    problems: Vec<Problem>,
}

#[derive(Deserialize)]
pub struct Problem {
    #[serde(rename = "contestId")]
    contest_id: u16,
    index: String,
    name: String,
    rating: Option<u16>,
}
