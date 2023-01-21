
use clap::{Parser, Subcommand, ColorChoice, value_parser, Args};
// use clap::{Parser, Subcommand, ValueEnum};
use owo_colors::{OwoColorize, Stream};
use anyhow::{anyhow, Result};
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};
use mime::Mime;

// 定义 HTTPie 的 CLI 的主入口，它包含若干个子命令
// 下面 /// 的注释是文档，clap 会将其作为 CLI 的帮助

/// A naive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Debug, Parser)]
#[command(version="1.0", author="Jack Xu")]
// #[command(setting = AppSettings::ColoredHelp)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(
        short,
        long,
        require_equals = true,
        value_name = "WHEN",
        num_args = 0..=2,
        default_value_t = ColorChoice::Auto,
        // default_missing_value = "auto",
        value_enum
    )]
    color: ColorChoice,
}

// 子命令分别对应不同的 HTTP 方法，目前只支持 get / post
#[derive(Debug, Subcommand)]
enum Command {
    Get(Get),
    Post(Post),
    // 我们暂且不支持其它 HTTP 方法
    // Get {
    //     #[arg(
    //         value_parser=value_parser!(Url)
    //     )]
    //     url: Url,
    // },
    // Post {
    //     // 是否为Json
    //     // todo
    //     /// HTTP 请求的 URL
    //     #[arg(
    //         value_parser=value_parser!(Url)
    //     )]
    //     url: Url,
    //     /// HTTP 请求的 body
    //     // #[value_parser(KvPair)]
    //     #[arg(
    //         value_parser=value_parser!(KvPair)
    //     )]
    //     body: Vec<KvPair>,
    // },
    
}

#[derive(Debug, Clone, Args)]
struct Get {
    #[arg(value_parser=value_parser!(Url))]
    url: Url,
}

#[derive(Debug, Clone, Args)]
struct Post {
    // 是否为Json
    // todo
    /// HTTP 请求的 URL
    #[arg(value_parser=value_parser!(Url))]
    url: Url,
    /// HTTP 请求的 body
    // #[value_parser(KvPair)]
    #[arg(value_parser=value_parser!(KvPair))]
    body: Vec<KvPair>,
}

#[derive(Debug, Clone)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

// fn parse_url(s: &str) -> Result<String> { 
//     // 这里我们仅仅检查一下 URL 是否合法 
//     let _url: Url = s.parse()?; 
//     Ok(s.into())
// }
// 
// fn parse_kv_pair(s: &str) -> Result<KvPair> {
//     Ok(s.parse()?)
// }

async fn get(client: Client, url: &Url) -> Result<()> {
    let resp = client.get(url.as_str()).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, url: &Url, args: &Vec<KvPair>) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(url.as_str()).json(&body).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

// 打印服务器版本号 + 状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status());
    println!("{}\n", status.if_supports_color(Stream::Stdout, |text| text.blue()));
}

// 打印服务器返回的 HTTP header
fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.if_supports_color(Stream::Stdout, |text| text.green()), value);
    }

    print!("\n");
}

/// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // 对于 "application/json" 我们 pretty print
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().if_supports_color(Stream::Stdout, |text| text.cyan()))
        }
        // 其它 mime type，我们就直接输出
        _ => println!("{}", body),
    }
}

/// 打印整个响应
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

/// 将服务器返回的 content-type 解析成 Mime 类型
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    // println!("{:?}", args);
    // init color
    match args.color {
        ColorChoice::Always => owo_colors::set_override(true),
        ColorChoice::Auto => {}
        ColorChoice::Never => owo_colors::set_override(false),
    }

    // 生成一个 HTTP 客户端 
    let client = Client::new();
    
    let result = match args.command {
        //Command::Get { url } => {
        Command::Get(ref cmd) => {
            // let parse_url = parse_url(url.as_str()).unwrap();
            println!(
                "GET {} (color={})", 
                cmd.url.if_supports_color(Stream::Stdout, |text| text.green()), 
                args.color
            );
            get(client, &cmd.url).await?
        },
        // Command::Post { url, body } => {
        Command::Post(ref cmd) => {
            // let parse_url = parse_url(url.as_str()).unwrap();
            println!(
                "POST {}, {:?} (color={})", 
                cmd.url.if_supports_color(Stream::Stdout, |text| text.bright_green()), 
                cmd.body.if_supports_color(Stream::Stdout, |text| text.bright_blue()), 
                args.color
            );
            post(client, &cmd.url, &cmd.body).await?
        }
    };
    Ok(result)
}