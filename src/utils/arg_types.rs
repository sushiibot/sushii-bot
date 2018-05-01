use std::sync::Arc;
use serenity::prelude::RwLock;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use serenity::utils::parse_channel;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandError;

/// Helper struct to get a Discord Channel from args
pub struct ChannelArg<'a> {
    args: &'a mut Args,
    // Whether or not to allow channels not in the guild
    allow_external: bool,
    err_invalid: String,
    err_external: String,
    guild: Option<Arc<RwLock<Guild>>>,
}

impl<'a> ChannelArg<'a> {
    pub fn new(args: &'a mut Args, guild: Option<Arc<RwLock<Guild>>>) -> Self {
        ChannelArg {
            args,
            allow_external: false,
            err_invalid: "Invalid channel.".into(),
            err_external: "Channel must be in this guild.".into(),
            guild,
        }
    }

    pub fn error(mut self, err: String) -> Self {
        self.err_invalid = err;

        self
    }

    pub fn error_external(mut self, err: String) -> Self {
        self.err_external = err;

        self
    }

    pub fn get(self) -> Result<u64, CommandError> {
        match self.args
            .single::<String>()
            .ok()
            .and_then(|x| x
                .parse::<u64>()
                .ok()
                .or(parse_channel(&x))
            ) {
            Some(channel) => {
                // if allow channels outside of this guild
                if self.allow_external {
                    return Ok(channel);
                }

                // check if in guild
                if let Some(guild) = self.guild {
                    let guild = guild.read();
                    if guild.channels.contains_key(&(ChannelId(channel))) {
                        return Ok(channel);
                    }

                    return Err(CommandError::from(self.err_external))
                }

                Err(CommandError::from(self.err_invalid))
            },
            None => Err(CommandError::from(self.err_invalid)),
        }
    }
}
