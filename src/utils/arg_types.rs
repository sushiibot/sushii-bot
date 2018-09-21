use std::sync::Arc;
use serenity::prelude::RwLock;
use serenity::model::guild::Guild;
use serenity::model::guild::Role;
use serenity::model::id::ChannelId;
use serenity::utils::parse_channel;
use serenity::utils::parse_role;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandError;

/// Helper struct to get a Discord Channel from args
pub struct ChannelArg<'a> {
    args: Option<&'a mut Args>,
    string: Option<String>,
    // Whether or not to allow channels not in the guild
    allow_external: bool,
    err_invalid: String,
    err_external: String,
    guild: Option<Arc<RwLock<Guild>>>,
}

impl<'a> Default for ChannelArg<'a> {
    fn default() -> Self {
        ChannelArg {
            args: None,
            string: None,
            allow_external: false,
            err_invalid: "Invalid channel.".into(),
            err_external: "Channel must be in this guild.".into(),
            guild: None,
        }
    }
}

impl<'a> ChannelArg<'a> {
    pub fn new() -> Self {
        ChannelArg::default()
    }

    /// Serenity Args struct
    pub fn args(mut self, args: &'a mut Args) -> Self {
        self.args = Some(args);

        self
    }

    /// Guild to check if the channel belongs to the guild
    pub fn guild(mut self, guild: Option<Arc<RwLock<Guild>>>) -> Self {
        self.guild = guild;

        self
    }

    /// String instead of Args struct
    pub fn string(mut self, string: &str) -> Self {
        self.string = Some(string.into());

        self
    }

    pub fn allow_external(mut self, should_allow: bool) -> Self {
        self.allow_external = should_allow;

        self
    }

    /// Custom error message for invalid channels
    pub fn error(mut self, err: String) -> Self {
        self.err_invalid = err;

        self
    }

    /// Custom error message for external channels
    pub fn error_external(mut self, err: String) -> Self {
        self.err_external = err;

        self
    }

    /// Parse and validate the channel
    pub fn get(self) -> Result<u64, CommandError> {
        let single_opt = if let Some(args) = self.args {
            args
                .single::<String>()
                .ok()
        } else {
            self.string
        };

        match single_opt
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

pub struct RoleArg<'a> {
    args: Option<&'a mut Args>,
    string: Option<String>,
    err_invalid: String,
    guild: Option<Arc<RwLock<Guild>>>,
}

impl<'a> Default for RoleArg<'a> {
    fn default() -> Self {
        RoleArg {
            args: None,
            string: None,
            err_invalid: "Invalid role.".into(),
            guild: None,
        }
    }
}

impl<'a> RoleArg<'a> {
    pub fn new() -> Self {
        RoleArg::default()
    }

    /// Serenity Args struct
    pub fn args(mut self, args: &'a mut Args) -> Self {
        self.args = Some(args);

        self
    }

    /// Guild to check if the role belongs to the guild
    pub fn guild(mut self, guild: Option<Arc<RwLock<Guild>>>) -> Self {
        self.guild = guild;

        self
    }

    /// String instead of Args struct
    pub fn string(mut self, string: &str) -> Self {
        self.string = Some(string.into());

        self
    }

    /// Custom error message for invalid roles
    pub fn error(mut self, err: String) -> Self {
        self.err_invalid = err;

        self
    }


    pub fn get(self) -> Result<Role, CommandError> {
        let single_opt = if let Some(args) = self.args {
            args
                .single::<String>()
                .ok()
        } else {
            self.string
        };

        match single_opt {
            Some(role) => {
                // check if in guild
                if let Some(guild) = self.guild {
                    let role_id = role.parse::<u64>()
                        .ok()
                        .or(parse_role(&role));
                    let guild = guild.read();

                    return guild.roles
                        .values()
                        .find(|x| {
                            if let Some(role_id) = role_id {
                                // check if role id matches if given arg is u64
                                x.id.0 == role_id
                            } else {
                                // check if role name matches if arg is string
                                x.name.to_lowercase() == role.to_lowercase()
                            }
                        })
                        .cloned()
                        .ok_or(CommandError::from(self.err_invalid));
                }

                Err(CommandError::from(self.err_invalid))
            },
            None => Err(CommandError::from(self.err_invalid)),
        }
    }
}
