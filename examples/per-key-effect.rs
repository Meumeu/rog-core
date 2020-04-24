use daemon::aura::{BuiltInModeByte, Key, KeyColourArray};
use daemon::daemon::{DBUS_IFACE, DBUS_NAME, DBUS_PATH};
use dbus::Error as DbusError;
use dbus::{ffidisp::Connection, Message};
use std::{thread, time};

pub fn dbus_led_builtin_write(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let bus = Connection::new_system()?;
    //let proxy = bus.with_proxy(DBUS_IFACE, "/", Duration::from_millis(5000));
    let msg = Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "ledmessage")?
        .append1(bytes.to_vec());
    let r = bus.send_with_reply_and_block(msg, 5000)?;
    if let Some(reply) = r.get1::<&str>() {
        println!("Success: {:x?}", reply);
        return Ok(());
    }
    Err(Box::new(DbusError::new_custom("name", "message")))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bus = Connection::new_system()?;

    let mut per_key_led = Vec::new();
    let mut key_colours = KeyColourArray::new();
    key_colours.set(Key::ROG, 255, 0, 0);
    key_colours.set(Key::L, 255, 0, 0);
    key_colours.set(Key::I, 255, 0, 0);
    key_colours.set(Key::N, 255, 0, 0);
    key_colours.set(Key::U, 255, 0, 0);
    key_colours.set(Key::X, 255, 0, 0);
    per_key_led.push(key_colours.clone());

    for _ in 0..85 {
        *key_colours.key(Key::ROG).0 -= 3;
        *key_colours.key(Key::L).0 -= 3;
        *key_colours.key(Key::I).0 -= 3;
        *key_colours.key(Key::N).0 -= 3;
        *key_colours.key(Key::U).0 -= 3;
        *key_colours.key(Key::X).0 -= 3;
        per_key_led.push(key_colours.clone());
    }
    for _ in 0..85 {
        *key_colours.key(Key::ROG).0 += 3;
        *key_colours.key(Key::L).0 += 3;
        *key_colours.key(Key::I).0 += 3;
        *key_colours.key(Key::N).0 += 3;
        *key_colours.key(Key::U).0 += 3;
        *key_colours.key(Key::X).0 += 3;
        per_key_led.push(key_colours.clone());
    }

    let ten_millis = time::Duration::from_millis(1);

    let row = KeyColourArray::get_init_msg();
    let msg =
        Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "ledmessage")?.append1(row);
    bus.send(msg).unwrap();

    loop {
        for group in &per_key_led {
            for row in group.get() {
                thread::sleep(ten_millis);
                let msg = Message::new_method_call(DBUS_NAME, DBUS_PATH, DBUS_IFACE, "ledmessage")?
                    .append1(row.to_vec());
                bus.send(msg).unwrap();
                // if let Some(reply) = r.get1::<&str>() {
                //     println!("Success: {:x?}", reply);
                //     return Ok(());
                // }
            }
        }
    }
}
