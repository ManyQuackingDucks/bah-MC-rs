#![deny(clippy::all)]
use lazy_static::lazy_static;

use reqwest::blocking;
use serde::Deserialize;
use serde::Serialize;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::{self, BufRead, Write};
use std::sync::RwLock;
use std::thread::{self};
use std::{net, vec};

const URL_BASE: &str = "https://api.hypixel.net/skyblock/auctions";

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[derive(Deserialize)]
struct Command {
    command: String,
    item: String,
    price: String,
    rarity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Item {
    item: String,
    price: i64,
    rarity: String,
}
#[derive(Serialize, Deserialize)]
struct Items {
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidItem {
    item: String,
    price: i64,
}

impl Item {
    fn new(item: String, price: i64, rarity: String) -> Self {
        Self {
            item,
            price,
            rarity,
        }
    }
}

lazy_static! {
    static ref ITEMS: RwLock<Vec<Item>> = RwLock::new({
        let mut m: Vec<Item> = vec![];
        let mut buf = String::new();
        let mut f = File::open("config.json").unwrap();
        f.read_to_string(&mut buf).unwrap();
        drop(f);
        let items: Items = serde_json::from_str(&buf).unwrap();
        for i in items.items {
            m.push(i);
        }
        m
    });
}

fn main() {
    //Exit if ANY thread panics
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));
    let stream = net::TcpStream::connect("127.0.0.1:6666").expect("Could not connect");

    let writestream = io::BufWriter::new(stream.try_clone().unwrap());

    thread::spawn(|| checkserver(writestream));
    let readstream = io::BufReader::new(stream.try_clone().unwrap());
    let f = File::open("config.json").unwrap();
    reciver(readstream, f);
    loop {
        thread::sleep(std::time::Duration::new(5, 0));
    }
}

fn reciver(mut readstream: io::BufReader<net::TcpStream>, mut f: File) {
    loop {
        let mut buffer = String::new();
        readstream
            .read_line(&mut buffer)
            .expect("Could not read from the data stream");
        let mut chars = buffer.chars();
        chars.next_back();
        chars.next_back(); //remove /r/n
        let command: Command = serde_json::from_str(&chars.as_str().to_owned()).unwrap();
        let mut lock = ITEMS.write().unwrap();
        match command.command.as_str() {
            "add" => {
                println!("adding");
                let item = Item::new(
                    command.item.replace('_', " "),
                    command.price.parse::<i64>().unwrap(),
                    command.rarity.to_ascii_uppercase(),
                );
                (*lock).push(item.clone());
                f.write_all(
                    &serde_json::to_vec(&Items {
                        items: (*lock).clone(),
                    })
                    .unwrap(),
                ).unwrap();
                println!("{:?}", item);
            }
            "del" => {
                println!("deleting");
                let item_name = command.item.replace('_', " ");
                (*lock).retain(|x| *x.item != item_name);
                f.write_all(
                    &serde_json::to_vec(&Items {
                        items: (*lock).clone(),
                    })
                    .unwrap(),
                ).unwrap();
                println!("{}", command.item);
            }
            _ => panic!("Invalid Command"), // Can not trigger unless I made a mistake in the java mod or I added a command in the java mod but not in here
        }
        drop(lock);
    }
}

fn checkserver(mut write_stream: io::BufWriter<net::TcpStream>) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap(); //Limit workers so it doesnt lag as bad
    for x in 1..num_cpus::get() {
        pool.spawn(move || println!("Initializing Worker {}", x))
    }
    let mut past_string = String::new();
    loop {
        let mut res = if let Ok(n) = blocking::get(URL_BASE) {
            n
        } else {
            thread::sleep(std::time::Duration::new(0, 500_000));
            blocking::get(URL_BASE).unwrap() //Connections may reset so wait 500 ms
        }
        .text()
        .unwrap();
        let first_page: serde_json::Value = simd_json::from_str(&mut res).unwrap();
        let total_pages = first_page["totalPages"].as_i64().unwrap();
        let mut threads = vec![];
        threads.reserve_exact(total_pages.try_into().unwrap());

        let (tx, rx) = std::sync::mpsc::channel();
        for x in 1..total_pages {
            let tx = tx.clone();
            pool.spawn(move || {
                tx.send(pagethread(x.try_into().unwrap()))
                    .expect("could not send");
            });
            threads.push(());
        }
        let read_lock = ITEMS.read().unwrap();
        let mut valid_items = vec![];
        for auction_item in first_page["auctions"].as_array().unwrap() {
            if let Some(auc_item) = auction_item.as_object() { //Sometimes returns None ?? This is defenitly a bug with simd_json because the line ablove is perfectly fine
                if auc_item.contains_key("bin") && !auc_item["claimed"].as_bool().unwrap() {
                    for i in &*read_lock {
                        if auc_item["item_name"].as_str().unwrap().contains(&i.item)
                            && auc_item["starting_bid"].as_i64().unwrap() <= i.price
                            && i.rarity == auc_item["tier"].as_str().unwrap()
                        {
                            valid_items.push(ValidItem {
                                item: i.item.clone(),
                                price: auc_item["starting_bid"].as_i64().unwrap(),
                            });
                        }
                    }
                }
            }
        }
        println!("Done with sort on main thread");
        drop(read_lock); //release read_lock as soon as posible
        drop(tx); //Will cause next line to block
        for thread in rx.into_iter().flatten() {
            for item in thread {
                valid_items.push(item);
            }
        }
        let mut sorted_items = vec![];
        while !valid_items.is_empty() {
            let mut simular = valid_items.clone();
            simular.retain(|x| x.item == valid_items[0].item);
            valid_items.retain(|x| x.item != simular[0].item);
            let lowest = simular.iter().min().unwrap();
            sorted_items.push(lowest.clone());
        }
        if !sorted_items.is_empty() {
            let mut send_string = "Found item\\s: ".to_string();
            for item in sorted_items {
                send_string.push_str(&format!("{} for {}, ", item.item, item.price));
            }
            let mut chars = send_string.chars();
            chars.next_back();
            chars.next_back();
            let mut send_string = chars.as_str().to_string();
            send_string.push('\r');
            send_string.push('\n');
            println!("{}", send_string);
            if send_string != past_string {
                write_stream
                    .write_all(send_string.as_bytes())
                    .expect("Could not write to stream");
                write_stream.flush().unwrap();
            }
            past_string = send_string;
        }
    }
}

fn pagethread(pagenum: usize) -> Option<Vec<ValidItem>> {
    let read_lock = ITEMS.read().unwrap();
    let mut res = if let Ok(n) = blocking::get(format!("{}?page={}", URL_BASE, pagenum)) {
        n
    } else {
        thread::sleep(std::time::Duration::new(0, 500_000));
        blocking::get(format!("{}?page={}", URL_BASE, pagenum)).unwrap()
    }
    .text()
    .unwrap();
    let page: serde_json::Value = simd_json::from_str(&mut res).unwrap();
    let mut valid_items = vec![];
    for auction_item in page["auctions"].as_array().unwrap() {
        if let Some(auc_item) = auction_item.as_object() {
            if auc_item.contains_key("bin") && !auc_item["claimed"].as_bool().unwrap() {
                for i in &*read_lock {
                    if auc_item["item_name"].as_str().unwrap().contains(&i.item)
                        && auc_item["starting_bid"].as_i64().unwrap() <= i.price
                        && i.rarity == auc_item["tier"].as_str().unwrap()
                    {
                        valid_items.push(ValidItem {
                            item: i.item.clone(),
                            price: auc_item["starting_bid"].as_i64().unwrap(),
                        });
                    }
                }
            }
        }
    }
    if valid_items.is_empty() {
        return None;
    }
    Some(valid_items)
}
