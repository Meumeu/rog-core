use rog_client::{AuraDbusWriter, GX502Layout, Key, KeyColourArray, KeyLayout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = AuraDbusWriter::new()?;

    let mut key_colours = KeyColourArray::new();
    let layout = GX502Layout::default();

    writer.init_effect()?;
    let rows = layout.get_rows();
    loop {
        for (r, row) in rows.iter().enumerate() {
            for (k, key) in row.iter().enumerate() {
                if let Some(c) = key_colours.key(*key) {
                    *c.0 = 255;
                };
                // Last key of previous row
                if k == 0 {
                    if r == 0 {
                        let k = &rows[rows.len() - 1][rows[rows.len() - 1].len() - 1];
                        if let Some(c) = key_colours.key(*k) {
                            *c.0 = 0;
                        };
                    } else {
                        let k = &rows[r - 1][rows[r - 1].len() - 1];
                        if let Some(c) = key_colours.key(*k) {
                            *c.0 = 0;
                        };
                    }
                } else {
                    let k = &rows[r][k - 1];
                    if let Some(c) = key_colours.key(*k) {
                        *c.0 = 0;
                    };
                }
                if let Some(c) = key_colours.key(Key::Up) {
                    *c.0 = 255;
                };
                *key_colours.key(Key::Left).unwrap().0 = 255;
                *key_colours.key(Key::Right).unwrap().0 = 255;
                *key_colours.key(Key::Down).unwrap().0 = 255;

                *key_colours.key(Key::W).unwrap().0 = 255;
                *key_colours.key(Key::A).unwrap().0 = 255;
                *key_colours.key(Key::S).unwrap().0 = 255;
                *key_colours.key(Key::D).unwrap().0 = 255;

                writer.write_colour_block(&key_colours)?;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}
