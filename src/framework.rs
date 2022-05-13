

pub struct FrameworkOptions {
    prefix: String,
    commands: Vec<Command>,
}

pub struct Command {
    name: String,
    description: String,
}

impl FrameworkOptions {
    pub fn new() -> FrameworkOptions {
        FrameworkOptions {
            prefix: "!".to_string(),
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn get_commands(&self) -> &Vec<Command> {
        &self.commands
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }
}