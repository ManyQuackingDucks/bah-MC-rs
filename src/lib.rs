#![no_main]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
use jni::objects::{JClass, JString, JValue};
use jni::JNIEnv;
use lazy_static::lazy_static;
use reqwest::blocking;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::{Mutex, RwLock};
use std::thread::{self};
use std::vec;
use std::fs::OpenOptions;

const CONFIG_PATH: &str = "config.json";
const URL_BASE: &str = "https://api.hypixel.net/skyblock/auctions";

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;


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

lazy_static! {
    static ref ITEMS: RwLock<Vec<Item>> = RwLock::new({
        let mut m: Vec<Item> = vec![];
        let mut buf = String::new();
        let mut f = File::open(CONFIG_PATH).unwrap();
        f.read_to_string(&mut buf).unwrap();
        drop(f);
        let items: Items = serde_json::from_str(&buf).unwrap();
        for i in items.items {
            m.push(i);
        }
        m
    });
    static ref CONFIG: Mutex<File> = Mutex::new(OpenOptions::new().read(false).write(true).open(CONFIG_PATH).unwrap());
}



#[no_mangle]
/// # Panics
///
/// Will panic if can not connect or invalid config.json
pub extern "system" fn Java_com_duck_bahmod_Server_rs_1start(env: JNIEnv, _: JClass){
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));
    let class = env.find_class("com/duck/bahmod/Server").expect("Could not find server class");
    checkserver(env, class);
}

fn send_chat(env: JNIEnv, class: JClass, msg: String){
    let output = env.new_string(msg).expect("Couldn't create java string!");
    let result = env.call_static_method(class, "MessageChat", "(Ljava/lang/String;)V", &[JValue::Object(output.into())]);
    result.map_err(|e| e.to_string()).unwrap();
}

#[no_mangle]
/// # Panics
///
/// Will panic if can not connect or invalid config.json
pub extern "system" fn Java_com_duck_bahmod_Server_add(env: JNIEnv, _:JClass, input:JString){
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();
    let mut item: Item = serde_json::from_str(&input).unwrap();
    let mut lock = ITEMS.write().unwrap();
    println!("adding");
    item.item = item.item.replace('_', " ");
    (*lock).push(item.clone());
    let mut conf_lock = CONFIG.lock().unwrap();
    (*conf_lock).write_all(
        &serde_json::to_vec_pretty(&Items {
            items: (*lock).clone(),
        })
        .unwrap(),
    ).unwrap();
    println!("{:?}", item);

}

#[no_mangle]
/// # Panics
///
/// Will panic if can not connect or invalid config.json
pub extern "system" fn Java_com_duck_bahmJava_com_duck_bahmod_Server_del(env: JNIEnv, _:JClass, input:JString){
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();
    let item: Item = serde_json::from_str(&input).unwrap();
    let mut lock = ITEMS.write().unwrap();
    println!("deleting");
    let item_name = item.item.replace('_', " ");
    (*lock).retain(|x| *x.item != item_name);
    let mut conf_lock = CONFIG.lock().unwrap();
    (*conf_lock).write_all(
        &serde_json::to_vec_pretty(&Items {
            items: (*lock).clone(),
        })
        .unwrap(),
    ).unwrap();
    println!("{}", item.item);
}


fn checkserver(env: JNIEnv, class: JClass) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap(); //Limit workers so it doesnt lag as bad
    for x in 1..num_cpus::get() {
        pool.spawn(move || println!("Initializing Worker {}", x));
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
        println!("Done with sort on main thread");
        let mut valid_items = sortpage(&first_page);
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
                send_chat(env, class, send_string.clone());
            }
            past_string = send_string;
        }
    }
}

fn sortpage(page: &Value) -> Vec<ValidItem>{
    let read_lock = ITEMS.read().unwrap();
    let mut valid_items = vec![];
    for auction_item in page["auctions"].as_array().unwrap() {
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
    valid_items
}

fn pagethread(pagenum: usize) -> Option<Vec<ValidItem>> {
    let mut res = if let Ok(n) = blocking::get(format!("{}?page={}", URL_BASE, pagenum)) {
        n
    } else {
        thread::sleep(std::time::Duration::new(0, 500_000));
        blocking::get(format!("{}?page={}", URL_BASE, pagenum)).unwrap()
    }
    .text()
    .unwrap();
    let page: serde_json::Value = simd_json::from_str(&mut res).unwrap();
    let valid_items = sortpage(&page);
    if valid_items.is_empty() {
        return None;
    }
    Some(valid_items)
}
