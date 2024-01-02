use std::{collections::HashMap, path};

use compiler::{build_binaries, build_std_lib, prep_objects::Context, Dictionaries};

pub fn bin(path: &str) {}

pub fn std() {
    println!("Loading std lib...");
    let ruda_path = match std::env::var("RUDA_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("RUDA_PATH not found. {}\nProject not compiled.", err);
            return;
        }
    };

    use compiler::*;
    let (ast, params, registry) = match generate_ast(&ruda_path) {
        Ok(ast) => (ast.ast, ast.params, ast.registry),
        Err(err) => {
            println!("{}", err);
            println!("Close all programs that use Ruda and try again.");
            println!("If that doesn't help, try to reinstall Ruda.");
            return;
        }
    };

    let mut temp_ast = (registry, Vec::new());
    let mut binaries = HashMap::new();
    let mut std_lib = match build_std_lib(&mut temp_ast) {
        Ok(std_lib) => std_lib,
        Err(err) => {
            println!("Failed to load std lib.");
            println!("{}", err);
            return;
        }
    };
    let mut dicts = Vec::new();
    let mut names = Vec::new();
    for _ in 0..std_lib.len() {
        let take = std_lib.remove(0);
        dicts.push(take.0);
        names.push(take.1);
    }
    drop(std_lib);
    match build_binaries(&Vec::new(), &mut temp_ast, &mut dicts) {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to load binaries.");
            println!("{}", err);
            return;
        }
    };
    for _ in 0..names.len() {
        binaries.insert(names.remove(0), dicts.remove(0));
    }

    println!("Std lib loaded.");
    println!("openning lens...");
    match open(Context::new(HashMap::new(), binaries)) {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to open lens.");
            println!("{:?}", err);
            return;
        }
    }
}

use iced::{
    executor,
    overlay::menu::State,
    theme::Text,
    widget::text,
    widget::{
        button, container, pick_list, scrollable, text_editor, Button, Column, PickList, Row,
    },
    window::Settings as WindowSettings,
    Application, Command, Element, Settings,
};

fn open(dict: Context) -> Result<(), LensErr> {
    match Lens::run(Settings::with_flags(LensFlags {
        objects: dict,
        project_name: String::from("STDLIB"),
    })) {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to open lens.");
            println!("{:?}", err);
            return Err(LensErr::CouldNotOpen);
        }
    };
    Ok(())
}

#[derive(Debug)]
pub enum LensErr {
    CouldNotOpen,
}

struct Lens {
    pub objects: Context,
    pub state: States,
    /// History of the lens
    /// has the form of (cursor, history)
    pub history: (usize, Vec<States>),
    pub project_name: String,
}

impl Lens {
    pub fn back(&mut self) {
        if self.history.0 == 0 {
            return;
        }
        self.history.0 -= 1;
        self.state = self.history.1[self.history.0].clone();
    }

    pub fn forward(&mut self) {
        if self.history.0 == self.history.1.len() - 1 {
            return;
        }
        self.history.0 += 1;
        self.state = self.history.1[self.history.0].clone();
    }
}

impl Application for Lens {
    fn new(flags: LensFlags) -> (Lens, iced::Command<Message>) {
        (
            Self {
                objects: flags.objects,
                state: States::Main,
                history: (0, Vec::new()),
                project_name: flags.project_name,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Ruda Lens")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match &message {
            Message::Page(page) => {
                self.history.1.truncate(self.history.0);
                self.history.0 += 1;
                self.history.1.push(self.state.clone());
                self.state = page.clone();
                iced::Command::none()
            }
            Message::Navigation(nav) => match nav {
                Navigation::Back => {
                    self.back();
                    iced::Command::none()
                }
                Navigation::Forward => {
                    self.forward();
                    iced::Command::none()
                }
            },
        }
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn style(&self) -> iced::theme::Application {
        iced::theme::Application::default()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let mut navigation = Row::new()
            .push(
                Button::new(text::Text::new("Back"))
                    .on_press(Message::Navigation(Navigation::Back)),
            )
            .push(
                Button::new(text::Text::new("Forward"))
                    .on_press(Message::Navigation(Navigation::Forward)),
            );
        let path = text::Text::new(format!(
            "Path: {}",
            self.state.into_path(&self.project_name)
        ));
        navigation = navigation.push(path);
        match &self.state {
            States::Main => {
                let files = {
                    let mut config = Row::new().spacing(10).push(text("Files: "));
                    for file in self.objects.0.keys() {
                        config = config.push(Button::new(text::Text::new(file)).on_press(
                            Message::Page(States::File(File {
                                name: file.clone(),
                                file_type: FileType::Rd,
                            })),
                        ));
                    }
                    Column::new().padding(10).push(config)
                };
                let dlls = {
                    let mut config = Row::new().spacing(10).push(text("Binaries: "));
                    for file in self.objects.1.keys() {
                        config = config.push(Button::new(text::Text::new(file)).on_press(
                            Message::Page(States::File(File {
                                name: file.clone(),
                                file_type: FileType::Dll,
                            })),
                        ));
                    }
                    Column::new().padding(10).push(config)
                };
                let config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(files)
                    .push(dlls);
                config.into()
            }
            States::File(file) => {
                let mut config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(text(format!("File: {}", file.name)).size(30));
                let dict = match file.file_type {
                    FileType::Rd => {
                        let dict = self.objects.0.get(&file.name).unwrap();
                        config = config.push(text(format!("Structs: {}", dict.structs.len())));
                        for obj in &dict.structs {
                            config = config.push(
                                Button::new(text::Text::new(&obj.identifier)).on_press(
                                    Message::Page(States::Struct {
                                        file: file.clone(),
                                        ident: obj.identifier.clone(),
                                    }),
                                ),
                            );
                        }
                        config = config.push(text(format!("Enums: {}", dict.enums.len())));
                        for obj in &dict.enums {
                            config = config.push(
                                Button::new(text::Text::new(&obj.identifier)).on_press(
                                    Message::Page(States::Enum {
                                        file: file.clone(),
                                        ident: obj.identifier.clone(),
                                    }),
                                ),
                            );
                        }
                        config = config.push(text(format!("Functions: {}", dict.functions.len())));
                        for obj in &dict.functions {
                            config = config.push(
                                Button::new(text::Text::new(
                                    obj.identifier.clone().unwrap_or("Anoymous".to_string()),
                                ))
                                .on_press(Message::Page(
                                    States::Function {
                                        file: file.clone(),
                                        ident: obj
                                            .identifier
                                            .clone()
                                            .unwrap_or("Anoymous".to_string())
                                            .clone(),
                                        block: None,
                                    },
                                )),
                            );
                        }
                        config = config.push(text(format!("Traits: {}", dict.traits.len())));
                        for obj in &dict.traits {
                            config = config.push(
                                Button::new(text::Text::new(&obj.identifier)).on_press(
                                    Message::Page(States::Trait {
                                        file: file.clone(),
                                        ident: obj.identifier.clone(),
                                    }),
                                ),
                            );
                        }
                        config = config.push(text(format!("Errors: {}", dict.errors.len())));
                        for obj in &dict.errors {
                            config = config.push(
                                Button::new(text::Text::new(&obj.identifier)).on_press(
                                    Message::Page(States::Error {
                                        file: file.clone(),
                                        ident: obj.identifier.clone(),
                                    }),
                                ),
                            );
                        }
                        config = config.push(text(format!("Consts: {}", dict.constants.len())));
                        for obj in &dict.constants {
                            config = config.push(
                                Button::new(text::Text::new(&obj.identifier)).on_press(
                                    Message::Page(States::Error {
                                        file: file.clone(),
                                        ident: obj.identifier.clone(),
                                    }),
                                ),
                            );
                        }
                    }
                    FileType::Dll => {
                        let dict = self.objects.1.get(&file.name).unwrap();
                        config = config.push(text(format!("Userdata: {}", dict.user_data.len())));
                        for obj in &dict.user_data {
                            config = config.push(Button::new(text::Text::new(&obj.name)).on_press(
                                Message::Page(States::UserData {
                                    file: file.clone(),
                                    ident: obj.name.clone(),
                                }),
                            ));
                        }
                        config = config.push(text(format!("Functions: {}", dict.functions.len())));
                        for obj in &dict.functions {
                            config = config.push(
                                Button::new(text::Text::new(obj.name.clone())).on_press(
                                    Message::Page(States::Function {
                                        file: file.clone(),
                                        block: None,
                                        ident: obj.name.clone(),
                                    }),
                                ),
                            );
                        }
                        config = config.push(text(format!("Enums: {}", dict.enums.len())));
                        for obj in &dict.enums {
                            config = config.push(Button::new(text::Text::new(&obj.name)).on_press(
                                Message::Page(States::Enum {
                                    file: file.clone(),
                                    ident: obj.name.clone(),
                                }),
                            ));
                        }
                        config = config.push(text(format!("Traits: {}", dict.traits.len())));
                        for obj in &dict.traits {
                            config = config.push(Button::new(text::Text::new(&obj.name)).on_press(
                                Message::Page(States::Trait {
                                    file: file.clone(),
                                    ident: obj.name.clone(),
                                }),
                            ));
                        }
                        config = config.push(text(format!("Consts: {}", dict.consts.len())));
                        for obj in &dict.consts {
                            config = config.push(Button::new(text::Text::new(&obj.name)).on_press(
                                Message::Page(States::Error {
                                    file: file.clone(),
                                    ident: obj.name.clone(),
                                }),
                            ));
                        }
                    }
                };
                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Enum { file, ident } => {
                let variants = match file.file_type {
                    FileType::Rd => {
                        let dict = self.objects.0.get(&file.name).unwrap();
                        let enum_ = dict
                            .enums
                            .iter()
                            .find(|enum_| enum_.identifier == *ident)
                            .unwrap();
                        let mut config = Column::new().spacing(10);
                        for variant in &enum_.keys {
                            config = config.push(text(format!("{} = {}", variant.0, variant.1)));
                        }
                        config
                    }
                    FileType::Dll => {
                        let dict = self.objects.1.get(&file.name).unwrap();
                        let enum_ = dict
                            .enums
                            .iter()
                            .find(|enum_| enum_.name == *ident)
                            .unwrap();
                        let mut config = Column::new().spacing(10);
                        for variant in &enum_.variants {
                            config = config.push(text(format!("{} = {}", variant.0, variant.1)));
                        }
                        config
                    }
                };
                let title = text(format!("Enum: {}", ident));
                let config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(title)
                    .push(variants);
                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Struct { file, ident } => {
                let (fields, methods) = match file.file_type {
                    FileType::Rd => {
                        let dict = self.objects.0.get(&file.name).unwrap();
                        let struct_ = dict
                            .structs
                            .iter()
                            .find(|struct_| struct_.identifier == *ident)
                            .unwrap();
                        let mut fields = Column::new().spacing(10).push(text("Fields:"));
                        for field in &struct_.fields {
                            fields = fields.push(text(format!("{}: {:?}", field.0, field.1)));
                        }
                        let mut methods = Column::new().spacing(10).push(text("Methods:"));
                        for method in &struct_.functions {
                            methods = methods.push(
                                Button::new(text::Text::new(method.identifier.as_ref().unwrap()))
                                    .on_press(Message::Page(States::Function {
                                        file: file.clone(),
                                        block: Some(ident.clone()),
                                        ident: method.identifier.clone().unwrap(),
                                    })),
                            );
                        }
                        (fields, methods)
                    }
                    FileType::Dll => {
                        let dict = self.objects.1.get(&file.name).unwrap();
                        let struct_ = dict
                            .structs
                            .iter()
                            .find(|struct_| struct_.name == *ident)
                            .unwrap();
                        let mut fields = Column::new().spacing(10).push(text("Fields:"));
                        for field in &struct_.fields {
                            fields = fields.push(text(format!("{}: {:?}", field.0, field.1)));
                        }
                        let mut methods = Column::new().spacing(10).push(text("Methods:"));
                        for method in &struct_.methods {
                            methods =
                                methods.push(Button::new(text::Text::new(&method.name)).on_press(
                                    Message::Page(States::Function {
                                        file: file.clone(),
                                        block: Some(ident.clone()),
                                        ident: method.name.clone(),
                                    }),
                                ));
                        }
                        (fields, methods)
                    }
                };
                let mut config = Column::new().spacing(10).push(navigation);

                config = config.push(fields);
                config = config.push(methods);

                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Function { file, block, ident } => {
                let mut kind = String::from("Function");
                let (params, returns) = match file.file_type {
                    FileType::Rd => {
                        let fun = match block {
                            Some(block) => {
                                let dict = self.objects.0.get(&file.name).unwrap();
                                let struct_ = dict
                                    .structs
                                    .iter()
                                    .find(|struct_| struct_.identifier == *block)
                                    .unwrap();
                                let fun = struct_
                                    .functions
                                    .iter()
                                    .find(|fun| fun.identifier == Some(ident.clone()))
                                    .unwrap();
                                kind = match fun.takes_self {
                                    true => "Method".to_string(),
                                    false => "Static method".to_string(),
                                };
                                if ident == block {
                                    kind = "*Constructor*".to_string();
                                }
                                fun
                            }
                            None => {
                                let dict = self.objects.0.get(&file.name).unwrap();
                                dict.functions
                                    .iter()
                                    .find(|fun| fun.identifier == Some(ident.clone()))
                                    .unwrap()
                            }
                        };
                        let mut params = Column::new().spacing(10).push(text("Params:"));
                        for param in &fun.args {
                            params = params
                                .push(text(format!("{}: {:?}", param.identifier, param.kind)));
                        }
                        let returns = text(format!("Returns: {:?}", fun.return_type));
                        (params, returns)
                    }
                    FileType::Dll => {
                        let fun = match block {
                            Some(block) => {
                                let dict = self.objects.1.get(&file.name).unwrap();
                                let ud =
                                    dict.user_data.iter().find(|ud| ud.name == *block).unwrap();
                                let fun = ud
                                    .methods
                                    .iter()
                                    .find(|fun| fun.name == ident.clone())
                                    .unwrap();
                                kind = match fun.takes_self {
                                    true => "Method".to_string(),
                                    false => "Static method".to_string(),
                                };
                                if ident == block {
                                    kind = "*Constructor*".to_string();
                                }
                                fun
                            }
                            None => {
                                let dict = self.objects.1.get(&file.name).unwrap();
                                dict.functions
                                    .iter()
                                    .find(|fun| fun.name == ident.clone())
                                    .unwrap()
                            }
                        };
                        let mut params = Column::new().spacing(10).push(text("Params:"));
                        for param in &fun.args {
                            params = params.push(text(format!("{}: {:?}", param.0, param.1)));
                        }
                        let returns = text(format!("Returns: {:?}", fun.return_type));
                        (params, returns)
                    }
                };
                let mut config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(text(format!("{}: {}", kind, ident)));

                config = config.push(params);
                config = config.push(returns);

                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Trait { file, ident } => {
                let trait_ = match file.file_type {
                    FileType::Rd => {
                        let dict = self.objects.0.get(&file.name).unwrap();
                        dict.traits
                            .iter()
                            .find(|trait_| trait_.identifier == *ident)
                            .unwrap()
                    }
                    FileType::Dll => todo!(),
                };
                let mut config = Column::new().spacing(10);
                for method in &trait_.methods {
                    config = config.push(
                        Button::new(text::Text::new(method.identifier.as_ref().unwrap())).on_press(
                            Message::Page(States::Function {
                                file: file.clone(),
                                block: Some(ident.clone()),
                                ident: method.identifier.clone().unwrap(),
                            }),
                        ),
                    );
                }
                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::UserData { file, ident } => {
                let ud = self
                    .objects
                    .1
                    .get(&file.name)
                    .unwrap()
                    .user_data
                    .iter()
                    .find(|ud| ud.name == *ident)
                    .unwrap();
                let mut config = Column::new().spacing(10).push(navigation);

                let mut methods = Column::new().spacing(10).push(text("Methods:"));
                for method in &ud.methods {
                    methods = methods.push(Button::new(text::Text::new(&method.name)).on_press(
                        Message::Page(States::Function {
                            file: file.clone(),
                            block: Some(ident.clone()),
                            ident: method.name.clone(),
                        }),
                    ));
                }
                config = config.push(methods);

                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Error { file, ident } => todo!(),
        }
    }

    type Message = Message;

    type Executor = executor::Default;

    type Theme = iced::Theme;

    type Flags = LensFlags;
}

#[derive(Debug, Clone)]
enum Message {
    Page(States),
    Navigation(Navigation),
}

#[derive(Debug, Clone)]
enum Navigation {
    Back,
    Forward,
}

struct LensFlags {
    pub objects: Context,
    pub project_name: String,
}

#[derive(Debug, Clone)]
enum States {
    /// Main menu
    ///
    /// This is the first screen that the user sees.
    /// It contains list of all the files in the project.
    Main,
    /// File
    ///
    /// This is the screen that shows the contents of a file.
    /// It contains a list of all the objects in the file.
    File(File),
    Enum {
        file: File,
        ident: String,
    },
    Struct {
        file: File,
        ident: String,
    },
    Function {
        file: File,
        /// In case this is a method, this is the block that it is in.
        block: Option<String>,
        ident: String,
    },
    Trait {
        file: File,
        ident: String,
    },
    Error {
        file: File,
        ident: String,
    },
    UserData {
        file: File,
        ident: String,
    },
}

impl States {
    pub fn into_path(&self, project: &str) -> String {
        let main = format!("{}", project);
        match self {
            States::Main => main.to_string(),
            States::File(file) => format!("{}:/{}", main, file.name),
            States::Enum { file, ident } => format!("{}:/{}/{}", main, file.name, ident),
            States::Struct { file, ident } => format!("{}:/{}/{}", main, file.name, ident),
            States::Function { file, block, ident } => {
                if let Some(block) = block {
                    format!("{}:/{}/{}:{}", main, file.name, block, ident)
                } else {
                    format!("{}:/{}/{}", main, file.name, ident)
                }
            }
            States::Trait { file, ident } => format!("{}:/{}/{}", main, file.name, ident),
            States::Error { file, ident } => format!("{}:/{}/{}", main, file.name, ident),
            States::UserData { file, ident } => format!("{}:/{}/{}", main, file.name, ident),
        }
    }
}

#[derive(Debug, Clone)]
enum FileType {
    Rd,
    Dll,
}

#[derive(Debug, Clone)]
struct File {
    pub name: String,
    pub file_type: FileType,
}
