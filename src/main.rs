use reqwest;
use serde_json::Value;

const BASE_STORE_INFO_URL: &'static str = "https://store.steampowered.com/api/appdetails?appids=";
const BASE_APPLIST_URL: &'static str = "http://api.steampowered.com/ISteamApps/GetAppList/v0002/";

fn find_appid_by_name(all_games: &Vec<Value>, target_name: &str) -> Option<Vec<(i64, String)>> {
    let mut matching_appids = Vec::new();

    for game in all_games {
        if let (Some(appid), Some(name)) = (game["appid"].as_i64(), game["name"].as_str()) {
            if name.to_lowercase().contains(target_name.to_lowercase().as_str()) {
                matching_appids.push((appid, name.to_owned()));
            }
        }
    }
    if matching_appids.is_empty() {
        return None;
    }
    Some(matching_appids)
}

fn category_value_to_vec(categories: &Vec<Value>) -> Option<Vec<(String, i64)>> {
    let mut category_vec = Vec::new();

    for cat in categories {
        if let (Some(name), Some(id)) = (cat["description"].as_str(), cat["id"].as_i64()) {
            category_vec.push((name.to_owned(), id));
        }
    }
    if category_vec.is_empty() {
        return None;
    }
    Some(category_vec)
}

fn genre_value_to_vec(categories: &Vec<Value>) -> Option<Vec<(String, String)>> {
    let mut genre_vec = Vec::new();

    for cat in categories {
        if let (Some(name), Some(id)) = (cat["description"].as_str(), cat["id"].as_str()) {
            genre_vec.push((name.to_owned(), id.to_owned()));
        }
    }
    if genre_vec.is_empty() {
        return None;
    }
    Some(genre_vec)
}

fn package_value_to_vec(packages: &Vec<Value>) -> Option<Vec<(String, bool, String, String, i64, i64, String, i64)>> {
    let mut package_vec = Vec::new();

    for packet in packages {
        if let (Some(can_get_free_license), 
                Some(is_free_license), 
                Some(option_description), 
                Some(option_text), 
                Some(package_id), 
                Some(percent_savings), 
                Some(percent_savings_text), 
                Some(price_in_cents_with_discount)) = (packet["can_get_free_license"].as_str(), packet["is_free_license"].as_bool(), packet["option_description"].as_str(), packet["option_text"].as_str(), packet["packageid"].as_i64(), packet["percent_savings"].as_i64(), packet["percent_savings_text"].as_str(), packet["price_in_cents_with_discount"].as_i64()) {
            package_vec.push((can_get_free_license.to_owned(), is_free_license, option_description.to_owned(), option_text.to_owned(), package_id, percent_savings, percent_savings_text.to_owned(), price_in_cents_with_discount));
        }
    }
    if package_vec.is_empty() {
        return None;
    }
    Some(package_vec)
}

fn packet_value_to_vec(packets: &Vec<Value>) -> Option<Vec<(String, String, i64, String, String, String, String, &Vec<Value>)>> {
    let mut packet_vec = Vec::new();

    for packet in packets {
        packet_vec.push((packet["title"].as_str().expect("Error on title").to_owned(), packet["description"].as_str().expect("Error on description").to_owned(), packet["display_type"].as_i64().expect("Error on dispaly type"), packet["is_recurring_subscription"].as_str().expect("Error on reocurring").to_owned(), packet["name"].as_str().expect("Error on name").to_owned(), packet["save_text"].as_str().expect("Error on stext").to_owned(), packet["selection_text"].as_str().expect("Error on seltext").to_owned(), packet["subs"].as_array().expect("Error on subs")));
    }

    Some(packet_vec)
}

pub fn read_input(prompt: &str) -> String {
    use std::io::{self, Write};
    let mut buffer: String = String::new();
    print!("{} ", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_owned()
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let res = reqwest::blocking::get(BASE_APPLIST_URL)?;
        let applist: Result<Value, serde_json::Error> = serde_json::from_str(res.text()?.as_str());
        let applist_value = applist?;
        let all_games: Vec<Value> = serde_json::from_value(applist_value["applist"]["apps"].clone())?;
        let games = find_appid_by_name(&all_games, read_input("Game name: ").as_str()).unwrap();
        let mut i = 0;
        for game in &games {
            println!("[{}] AppId: {}, Name: {}", i, game.0, game.1);
            i = i + 1;
        }
        let index = read_input("Index: ").parse::<usize>()?;
        if index <= i {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            let actual_game = &games[index];
            println!("Game found: {} | {}", actual_game.1, actual_game.0);
            let store_response = reqwest::blocking::get(format!("{}{}", BASE_STORE_INFO_URL, actual_game.0.to_owned()))?;
            let details: Value = serde_json::from_str(store_response.text()?.as_str())?;
            //println!("Details: {}", serde_json::to_string_pretty(&details[format!("{}", &actual_game.0)]).expect("Failed to convert to json str"));
            let game_details: &Value = &details[format!("{}", &actual_game.0)]["data"];
        
            let game_description: String = serde_json::to_string(&game_details["short_description"])?;
            let categories: Vec<Value> = serde_json::from_value(game_details["categories"].clone())?;
            println!("Categories:");
            for category in category_value_to_vec(&categories).expect("Failed to convert value to vec!(category)") {
                print!("{}, ", category.0);
            }
            let genres: Vec<Value> = serde_json::from_value(game_details["genres"].clone())?;
            println!("Genres:");
            for genre in genre_value_to_vec(&genres).expect("Failed to convert value to vec!(genre)") {
                print!("{}, ", genre.0);
            }
            println!("Game description: {}", game_description);
            let package_groups: Vec<Value> = serde_json::from_value(game_details["package_groups"].clone())?;
            println!("Package Groups:");
            for packet in packet_value_to_vec(&package_groups).expect("Failed to convert value to vec!(package_groups)") {
                println!("Packages:");
                for package in package_value_to_vec(&packet.7).expect("Failed to convert value to vec!(package)") {
                    println!("Name: {} | Desc: {} | Free: {} | Can: {}", package.3, package.2, package.1, package.0);
                    println!("Savings/Price: %Savings: {} | %Savings Text: {} | Price in cents with discounts: {}", package.5, package.6, package.7);
                }
            }
            println!("Image: https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg", actual_game.0);
            println!("Steam Link: https://store.steampowered.com/app/{}/", actual_game.0);
        } else {
            panic!("Index was out of bounds: {}/{}", index, i);
        }
    }
    Ok(())
}
