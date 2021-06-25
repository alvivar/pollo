pub struct Op {
    pub op: String,
    pub key: String,
    pub value: String,
}

pub fn parse(text: &str) -> Op {
    let mut op = String::new();
    let mut key = String::new();
    let mut value = String::new();

    let mut deep = 0;
    for c in text.chars() {
        match c {
            ' ' => {
                if value.len() > 0 {
                    value.push(' ');
                } else if key.len() > 0 {
                    deep = 2;
                } else if op.len() > 0 {
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

    let op = op.trim().to_owned();
    let key = key.trim().to_owned();

    Op { op, key, value }
}
