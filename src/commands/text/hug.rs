use serenity::framework::standard::CommandError;

use rand::{thread_rng, Rng};

const HUGS_LEFT: &'static [&'static str] = &[
    "ლ(・ヮ・ლ)",
    "⊂(・﹏・⊂)",
    "⊂(・ヮ・⊂)",
    "⊂(・▽・⊂)",
    "ლ(・﹏・ლ)",
    "⊂(･ω･*⊂)",
    "ლ(･ω･*ლ)",
    "ლ(´ ❥ `ლ)",
    "⊂(´・ω・｀⊂)",
];

const HUGS_RIGHT: &'static [&'static str] = &[
    "⊂(◉‿◉)つ",
    "(つ◉益◉)つ",
    "(っಠ‿ಠ)っ",
    "ʕっ•ᴥ•ʔっ",
    "(っ・∀・）っ",
    "(っ⇀⑃↼)っ",
    "(つ´∀｀)つ",
    "(つ▀¯▀)つ",
    "(っ´▽｀)っ",
    "(づ￣ ³￣)づ",
    "c⌒っ╹v╹ )っ",
    "(.づ◡﹏◡)づ.",
    "(っ*´∀｀*)っ",
    "(っ⇀`皿′↼)っ",
    "(.づσ▿σ)づ.",
];

command!(hug_cmd(_ctx, msg, args) {
    let target = match args.single::<String>() {
        Ok(val) => val,
        Err(_) => return Err(CommandError::from(get_msg!("error/hug_nobody"))),
    };

    let mut rng = thread_rng();
    
    // alternate between right and left?
    let hug = if msg.id.0 % 2 == 0 {
        format!("{} {}", target, rng.choose(&HUGS_LEFT).unwrap())
    } else {
        format!("{} {}", rng.choose(&HUGS_RIGHT).unwrap(), target)
    };

    // clean hug from everyone and here mentions just in case
    let hug = hug
        .replace("@everyone", "@\u{200b}everyone")
        .replace("@here", "@\u{200b}here");

    let _ = msg.channel_id.say(&hug);
});
