use std::{
    cell::RefCell,
    collections::HashMap,
    env::{args, Args},
    fmt::Write,
    iter::{Peekable, Skip},
    ops::Deref,
    path::MAIN_SEPARATOR,
    rc::Rc,
};

const BOLDGREEN: &str = "\x1b[1m\x1b[32m";
const BOLDYELLOW: &str = "\x1b[1m\x1b[33m";
const BOLDCYAN: &str = "\x1b[1m\x1b[36m";
const RESET: &str = "\x1b[0m";

#[derive(Debug)]
pub enum ArgumentParserError {
    MissingArgument(String),
    InvalidArgument(String),
    Fmt(String),
    Parse(String),
}

impl std::fmt::Display for ArgumentParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentParserError::MissingArgument(msg) => write!(f, "Missing argument: {}", msg),
            ArgumentParserError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            ArgumentParserError::Fmt(msg) => write!(f, "Fmt error: {}", msg),
            ArgumentParserError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl From<std::fmt::Error> for ArgumentParserError {
    fn from(err: std::fmt::Error) -> Self {
        ArgumentParserError::Fmt(err.to_string())
    }
}

impl std::error::Error for ArgumentParserError {}

type Result<T> = std::result::Result<T, ArgumentParserError>;

#[derive(Debug)]
pub struct PositionalArgument {
    name: String,
    description: String,
    value: Option<String>,
}

impl PositionalArgument {
    pub fn new(name: &str, description: Option<&str>) -> PositionalArgument {
        PositionalArgument {
            name: name.to_string(),
            description: description.unwrap_or("").to_string(),
            value: None,
        }
    }
}

#[derive(Debug)]
pub struct OptionalArgument {
    name: String,
    short_name: String,
    description: String,
    value: Option<String>,
}

impl OptionalArgument {
    pub fn new(name: &str, short_name: Option<&str>, description: Option<&str>) -> OptionalArgument {
        OptionalArgument {
            name: name.to_string(),
            short_name: short_name.unwrap_or("").to_string(),
            description: description.unwrap_or("").to_string(),
            value: None,
        }
    }
}

#[derive(Debug)]
pub struct FlagArgument {
    name: String,
    short_name: String,
    description: String,
    value: bool,
}

impl FlagArgument {
    pub fn new(name: &str, short_name: Option<&str>, description: Option<&str>) -> FlagArgument {
        FlagArgument {
            name: name.to_string(),
            short_name: short_name.unwrap_or("").to_string(),
            description: description.unwrap_or("").to_string(),
            value: false,
        }
    }
}

#[derive(Debug)]
pub enum Argument {
    Optional(OptionalArgument),
    Flag(FlagArgument),
}

#[derive(Debug)]
pub struct ArgumentParser {
    program_name: String,
    program_description: String,
    positionals_count: usize,
    positionals: HashMap<usize, PositionalArgument>,
    optionals: HashMap<String, Rc<RefCell<Argument>>>,
}

impl ArgumentParser {
    pub fn new() -> ArgumentParser {
        let program_name = args()
            .next()
            .unwrap_or("MyProgram".to_string())
            .rsplit_once(MAIN_SEPARATOR)
            .unwrap_or(("", "MyProgram"))
            .1
            .to_string();

        ArgumentParser {
            program_name,
            program_description: "".to_string(),
            positionals_count: 0,
            positionals: HashMap::new(),
            optionals: HashMap::new(),
        }
    }

    pub fn add_positional(&mut self, name: &str, description: Option<&str>) {
        self.positionals
            .entry(self.positionals_count)
            .or_insert(PositionalArgument::new(name, description));

        self.positionals_count += 1;
    }

    pub fn add_optional(&mut self, name: &str, short_name: Option<&str>, description: Option<&str>) -> Result<()> {
        let short_name_owned = match short_name {
            Some(s) => s.to_string(),
            None => {
                let first_char = name
                    .chars()
                    .find(|c| *c != '-')
                    .ok_or_else(|| ArgumentParserError::InvalidArgument("Failed to infer short name of ".to_string() + name))?;

                format!("-{}", first_char)
            }
        };

        let opt_arg = Rc::new(RefCell::new(Argument::Optional(OptionalArgument::new(
            name,
            Some(short_name_owned.as_str()),
            description,
        ))));

        self.optionals.insert(name.to_string(), Rc::clone(&opt_arg));
        self.optionals.insert(short_name_owned, Rc::clone(&opt_arg));
        Ok(())
    }

    pub fn add_flag(&mut self, name: &str, short_name: Option<&str>, description: Option<&str>) -> Result<()> {
        let short_name_owned = match short_name {
            Some(s) => s.to_string(),
            None => {
                let first_char = name
                    .chars()
                    .find(|c| *c != '-')
                    .ok_or_else(|| ArgumentParserError::InvalidArgument("Failed to infer short name of ".to_string() + name))?;

                format!("-{}", first_char)
            }
        };

        let flag = Rc::new(RefCell::new(Argument::Flag(FlagArgument::new(
            name,
            Some(short_name_owned.as_str()),
            description,
        ))));

        self.optionals.insert(name.to_string(), Rc::clone(&flag));
        self.optionals.insert(short_name_owned, Rc::clone(&flag));

        Ok(())
    }

    pub fn positional(&self, name: &str) -> Result<String> {
        match self
            .positionals
            .iter()
            .find(|(_, arg)| arg.name == name)
            .and_then(|(_, arg)| arg.value.clone())
        {
            Some(value) => Ok(value),
            None => Err(ArgumentParserError::MissingArgument(format!(
                "Missing positional argument: {}",
                name
            ))),
        }
    }

    pub fn optional(&self, name: &str) -> Result<String> {
        for (_, arg_rc) in self.optionals.iter() {
            let Argument::Optional(opt_arg) = &*arg_rc.borrow() else {
                continue;
            };

            if opt_arg.name == name || opt_arg.short_name == name {
                return match opt_arg.value.clone() {
                    Some(value) => Ok(value),
                    None => Err(ArgumentParserError::MissingArgument(format!(
                        "Missing value for optional argument: {}, {}",
                        opt_arg.name, opt_arg.short_name
                    ))),
                };
            }
        }

        Err(ArgumentParserError::InvalidArgument(format!(
            "Unrecognized optional argument: {}",
            name
        )))
    }

    pub fn flag(&self, name: &str) -> Result<bool> {
        for (_, arg_rc) in self.optionals.iter() {
            let Argument::Flag(flag_arg) = &*arg_rc.borrow() else {
                continue;
            };

            if flag_arg.name == name || flag_arg.short_name == name {
                return Ok(flag_arg.value);
            }
        }

        Err(ArgumentParserError::InvalidArgument(format!(
            "Unrecognized flag: {}",
            name
        )))
    }

    pub fn set_program_description(&mut self, description: &str) {
        self.program_description = description.to_string();
    }

    pub fn set_program_name(&mut self, name: &str) {
        self.program_name = name.to_string();
    }

    pub fn help(&self) -> Result<()> {
        let mut buffer: String = String::with_capacity(
            self.program_name.len() + self.program_description.len() + self.positionals_count * 100 + self.optionals.len() * 200,
        );

        writeln!(
            &mut buffer,
            "{}{}\n{}{}",
            BOLDGREEN,
            self.program_name
                .rsplit_once('.')
                .unwrap_or((&self.program_name, ""))
                .0,
            self.program_description,
            RESET
        )?;

        write!(buffer, "Usage: {}{}{}", BOLDGREEN, self.program_name, RESET)?;

        write!(buffer, "{}", BOLDYELLOW)?;
        for i in 0..self.positionals_count {
            if let Some(pos_arg) = self.positionals.get(&i) {
                write!(&mut buffer, " <{}> ", pos_arg.name)?;
            }
        }
        write!(buffer, "{}", RESET)?;

        writeln!(&mut buffer, "{}[OPTIONS]{}\n", BOLDCYAN, RESET)?;

        writeln!(buffer, "\tPositional arguments:\n")?;
        for i in 0..self.positionals_count {
            if let Some(pos_arg) = self.positionals.get(&i) {
                writeln!(
                    &mut buffer,
                    "\t\t{}{}{}:\n\t\t\t{}",
                    BOLDYELLOW, pos_arg.name, RESET, pos_arg.description
                )?;
            }
        }

        write!(buffer, "\n\t")?;
        writeln!(buffer, "Optional arguments:\n")?;

        for (_, o) in self.optionals.iter() {
            match o.borrow().deref() {
                Argument::Optional(opt_arg) => {
                    if !buffer.contains(&opt_arg.name) {
                        write!(
                            &mut buffer,
                            "\t\t{}{} ({}){}:\n\t\t\t{}\n",
                            BOLDCYAN, opt_arg.name, opt_arg.short_name, RESET, opt_arg.description
                        )?;
                    }
                }
                Argument::Flag(flag_arg) => {
                    if !buffer.contains(&flag_arg.name) {
                        write!(
                            &mut buffer,
                            "\t\t{}{} ({}){}:\n\t\t\t{}\n",
                            BOLDCYAN, flag_arg.name, flag_arg.short_name, RESET, flag_arg.description
                        )?;
                    }
                }
            }
        }

        println!("{}", buffer);

        Ok(())
    }

    fn handle_optional_argument(&mut self, arg: &str, args: &mut Peekable<Skip<Args>>, positional_index: usize) -> Result<()> {
        if positional_index < self.positionals_count {
            if let Some(pos_arg) = self.positionals.get(&positional_index) {
                return Err(ArgumentParserError::Parse(format!(
                    "Expected positional argument: {}",
                    pos_arg.name
                )));
            }
        }

        let argument_rc = match self.optionals.get(arg) {
            Some(arg_rc) => arg_rc,
            None => {
                return Err(ArgumentParserError::Parse(format!(
                    "Unrecognized optional argument: {}",
                    arg
                )));
            }
        };

        let mut argument = argument_rc.borrow_mut();

        let next_arg = match args.peek() {
            Some(next_arg) => next_arg,
            None => &"".to_string(),
        };

        match &mut *argument {
            Argument::Optional(opt_arg) => {
                if !next_arg.starts_with('-') && !next_arg.is_empty() {
                    opt_arg.value = Some(args.next().unwrap());
                    Ok(())
                } else {
                    Err(ArgumentParserError::Parse(format!(
                        "Expected value for optional argument: {}",
                        opt_arg.name
                    )))
                }
            }
            Argument::Flag(flag_arg) => {
                flag_arg.value = true;
                Ok(())
            }
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        let mut args = args().skip(1).peekable();
        let mut positional_index = 0;

        while let Some(arg) = args.next() {
            if arg == "-h" || arg == "--help" {
                self.help()?;
                return Ok(());
            }

            if arg.starts_with('-') {
                self.handle_optional_argument(&arg, &mut args, positional_index)?
            } else if let Some(pos_arg) = self.positionals.get_mut(&positional_index) {
                pos_arg.value = Some(arg);
                positional_index += 1;
            } else {
                return Err(ArgumentParserError::Parse(format!(
                    "Unrecognized positional argument: {}",
                    arg
                )));
            }
        }
        Ok(())
    }
}
