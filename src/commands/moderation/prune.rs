use serenity::framework::standard::CommandError;
use serenity::model::MessageId;

command!(prune(_ctx, msg, args) {
    let count = match args.single::<u64>() {
        Ok(val) => val + 1,
        Err(_) => return Err(CommandError::from(get_msg!("error/invalid_number"))),
    };

    // validate input
    if count < 1 || count > 99 {
        return Err(CommandError::from(get_msg!("error/prune_too_high")));
    }

    let messages = match msg.channel_id.messages(|x| x.limit(count)) {
        Ok(val) => val.iter().map(|x| x.id).collect::<Vec<MessageId>>(),
        Err(_) => return Err(CommandError::from(get_msg!("error/failed_get_messages"))),
    };

    if let Err(_) = msg.channel_id.delete_messages(messages) {
        return Err(CommandError::from(get_msg!("error/failed_delete_messages")));
    }
});
