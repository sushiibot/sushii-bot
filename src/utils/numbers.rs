pub fn comma_number(num: u64) -> String {
    let string = num.to_string();
    let mut output = String::new();

    let mut place = string.len();
    let mut later_loop = false;

    for ch in string.chars() {
        if later_loop && place % 3 == 0 {
            output.push(',');
        }

        output.push(ch);
        later_loop = true;
        place -= 1;
    };

    output
}