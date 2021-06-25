pub struct Msg {
    pub op: String,
    pub key: String,
    pub value: String,
}

pub fn parse(text: &str) -> Msg {
    let mut op = String::new();
    let mut key = String::new();
    let mut value = String::new();

    let mut deep = 0;
    for c in text.chars() {
        match c {
            ' ' => {
                if !value.is_empty() {
                    value.push(' ');
                } else if !key.is_empty() {
                    deep = 2;
                } else if !op.is_empty() {
                    deep = 1;
                }
            }

            _ => match deep {
                0 => {
                    op.push(c);
                }
                1 => {
                    key.push(c);
                }
                _ => {
                    value.push(c);
                }
            },
        }
    }

    let op = op.trim_end().to_owned();
    let key = key.trim().to_owned();

    Msg { op, key, value }
}
