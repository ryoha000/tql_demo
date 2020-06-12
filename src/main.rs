extern crate chrono;
extern crate postgres;
#[macro_use]
extern crate tql;
#[macro_use]
extern crate tql_macros;

use std::env;

use chrono::DateTime;
use chrono::offset::Utc;
use postgres::{Connection, TlsMode};
use tql::PrimaryKey;

// A TodoItem is a table containing a text, an added date and a done boolean.
#[derive(SqlTable)]
struct TodoItem {
    id: PrimaryKey,
    text: String,
    date_added: DateTime<Utc>,
    done: bool,
}

fn add_todo_item(connection: Connection, text: String) {
    // Insert the new item.
    let date_added = Utc::now(); // FIXME: remove the variable when function-like proc-macro works on stable.
    let result = sql!(connection, TodoItem.insert(text = text, date_added = date_added, done = false));
    if let Err(err) = result {
        println!("Failed to add the item ({})", err);
    }
    else {
        println!("Item added");
    }
}

fn delete_todo_item(connection: Connection, id: i32) {
    // Delete the item.
    let result = sql!(connection, TodoItem.get(id).delete());
    if let Err(err) = result {
        println!("Failed to delete the item ({})", err);
    }
    else {
        println!("Item deleted");
    }
}

fn do_todo_item(cx: Connection, id: i32) {
    // Update the item to make it done.
    let result = sql!(cx, TodoItem.get(id).update(done = true));
    if let Err(err) = result {
        println!("Failed to do the item ({})", err);
    }
    else {
        println!("Item done");
    }
}

fn get_id(args: &mut env::Args) -> Option<i32> {
    if let Some(arg) = args.next() {
        if let Ok(id) = arg.parse() {
            return Some(id);
        }
        else {
            println!("Please provide a valid id");
        }
    }
    else {
        println!("Missing argument: id");
    }
    None
}

fn list_todo_items(connection: &Connection, show_done: bool) -> Result<(), ::postgres::Error> {
    let items =
        if show_done {
            // Show the last 10 todo items.
            sql!(connection, TodoItem.sort(-date_added)[..10])?
        }
        else {
            // Show the last 10 todo items that are not done.
            sql!(connection, TodoItem.filter(done == false).sort(-date_added)[..10])?
        };

    for item in items {
        let done_text =
            if item.done {
                "(✓)"
            }
            else {
                ""
            };
        println!("{}. {} {}", item.id, item.text, done_text);
    }

    Ok(())
}

fn main() {
    let connection = get_connection();

    // Create the table.
    let _ = sql!(connection, TodoItem.create());

    let mut args = env::args();
    args.next();

    let command = args.next().unwrap_or("list".to_string());
    match command.as_ref() {
        "add" => {
            if let Some(item_text) = args.next() {
                add_todo_item(connection, item_text);
            }
            else {
                println!("Missing argument: task");
            }
        },
        "delete" => {
            if let Some(id) = get_id(&mut args) {
                delete_todo_item(connection, id);
            }
        },
        "do" => {
            if let Some(id) = get_id(&mut args) {
                do_todo_item(connection, id);
            }
        },
        "list" => {
            let show_done = args.next() == Some("--show-done".to_string());
            list_todo_items(&connection, show_done)
                .expect("Cannot fetch todo items");
        },
        command => println!("Unknown command {}", command),
    }
}

fn get_connection() -> Connection {
    Connection::connect("postgres://root:password@localhost:5433/test", TlsMode::None).unwrap()
}