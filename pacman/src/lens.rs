use std::{collections::HashMap, path};

use compiler::{build_binaries, build_std_lib, prep_objects::Context, Dictionaries};

/// Reads the projects output binary and opens lens for it
pub fn bin(path: &str, profile: (&str, &config::Profile)) {
    // check if there is directory for the profile
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if !profile_path.exists() {
        // create directory
        std::fs::create_dir_all(&profile_path).unwrap();
    }
    let ruda_path = match std::env::var("RUDA_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("RUDA_PATH not found. {}\nProject not compiled.", err);
            return;
        }
    };
    let binary = match profile.1.kind {
        config::ProjectKind::Bin => {
            let bin_path = std::path::Path::new(path)
                .join("target")
                .join(profile.0)
                .join("out.rdbin");
            bin_path
        }
        config::ProjectKind::Lib => {
            let lib_path = std::path::Path::new(path)
                .join("target")
                .join(profile.0)
                .join("out.rdlib");
            lib_path
        }
    };
    let str = match std::fs::read_to_string(&binary) {
        Ok(string) => string,
        Err(err) => {
            println!("Failed to read binary.");
            println!("{}", err);
            return;
        }
    };
    let bin = stringify::parse(&str);
    println!("Binary loaded: {:?}", bin);
    // open bin lens
    match bin_lens::BinLens::run(Settings::with_flags(BinLensFlags {
        objects: bin,
        project_name: profile.0.to_string(),
    })) {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to open lens.");
            println!("{:?}", err);
            return;
        }
    };
}

pub fn project(path: &str, profile: (&str, &config::Profile)) {
    // check if there is directory for the profile
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if !profile_path.exists() {
        // create directory
        std::fs::create_dir_all(&profile_path).unwrap();
    }
    let ruda_path = match std::env::var("RUDA_PATH") {
        Ok(path) => path,
        Err(err) => {
            println!("RUDA_PATH not found. {}\nProject not compiled.", err);
            return;
        }
    };
    let main_file = match profile.1.kind {
        config::ProjectKind::Bin => {
            let bin_path = std::path::Path::new(path).join("src").join("main.rd");
            bin_path
        }
        config::ProjectKind::Lib => {
            let lib_path = std::path::Path::new(path).join("src").join("lib.rd");
            lib_path
        }
    };
    let main_file = match main_file.to_str() {
        Some(file) => file,
        None => {
            println!("Failed to convert path to string.");
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
    //println!("AST generated.");
    let dictionaries = match build_dictionaries(&main_file, &mut (ast, params)) {
        Ok(dictionaries) => dictionaries,
        Err(err) => {
            println!("Failed to load dictionaries.");
            println!("Err: '{}':{}", err.1, err.0);
            return;
        }
    };
    //println!("Dictionary generated.");
    // println!("{:?}", dictionaries);
    // BEWARE: this part is what you call a technical debt
    let mut bin_paths = Vec::new();
    let mut lib_names = Vec::new();
    for (lib_name, lib_path) in &profile.1.binaries {
        let lib_path = std::path::Path::new(path).join(lib_path);
        if !lib_path.exists() {
            println!("{} does not exist.", lib_path.to_str().unwrap());
            return;
        }
        let lib_path = match lib_path.to_str() {
            Some(path) => path,
            None => {
                println!("Failed to convert path to string.");
                return;
            }
        };
        bin_paths.push(lib_path.to_string());
        lib_names.push(lib_name.to_string());
    }
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
    match build_binaries(&bin_paths, &mut temp_ast, &mut dicts) {
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
    for (_, libname) in lib_names.iter().enumerate() {
        binaries.insert(libname.to_string(), dicts.remove(0));
    }
    const LIB_COUNT: usize = 9;
    const STD_LIBS: [&str; LIB_COUNT] = [
        "#io", "#string", "#fs", "#algo", "#core", "#time", "#window", "#memory", "#math",
    ];
    let mut count = LIB_COUNT;
    for (name, bin) in binaries.iter_mut() {
        match STD_LIBS.iter().position(|&lib| lib == name) {
            Some(idx) => {
                bin.id = idx;
            }
            None => {
                bin.id = count;
                count += 1;
            }
        }
    }

    let mut context = Context::new(dictionaries, binaries);
    match prep_objects::prep(&mut context) {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to prepare objects.");
            // TODO: println!("{}", err);
            return;
        }
    }

    match open(context, path.to_string()) {
        Ok(()) => {}
        Err(err) => {
            println!("Failed to open lens.");
            println!("{:?}", err);
            return;
        }
    }
}

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
    match open(
        Context::new(HashMap::new(), binaries),
        "*STDLIB*".to_string(),
    ) {
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
use stringify::Data;

use crate::{config, lens::bin_lens::BinLensFlags};

fn open(dict: Context, project_name: String) -> Result<(), LensErr> {
    match Lens::run(Settings::with_flags(LensFlags {
        objects: dict,
        project_name,
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
        if self.history.0 + 1 >= self.history.1.len() {
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
                        if dict.structs.len() > 0 {
                            config = config.push(text(format!("Structs: {}", dict.structs.len())));
                            for obj in &dict.structs {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.identifier)).on_press(
                                        Message::Page(States::Struct {
                                            file: file.clone(),
                                            ident: obj.identifier.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.enums.len() > 0 {
                            config = config.push(text(format!("Enums: {}", dict.enums.len())));
                            for obj in &dict.enums {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.identifier)).on_press(
                                        Message::Page(States::Enum {
                                            file: file.clone(),
                                            ident: obj.identifier.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.functions.len() > 0 {
                            config =
                                config.push(text(format!("Functions: {}", dict.functions.len())));
                            for obj in &dict.functions {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(obj.identifier.as_ref().unwrap()))
                                        .on_press(Message::Page(States::Function {
                                            file: file.clone(),
                                            block: None,
                                            ident: obj.identifier.clone().unwrap(),
                                        })),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.traits.len() > 0 {
                            config = config.push(text(format!("Traits: {}", dict.traits.len())));
                            for obj in &dict.traits {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.identifier)).on_press(
                                        Message::Page(States::Trait {
                                            file: file.clone(),
                                            ident: obj.identifier.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.errors.len() > 0 {
                            config = config.push(text(format!("Errors: {}", dict.errors.len())));
                            for obj in &dict.errors {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.identifier)).on_press(
                                        Message::Page(States::Error {
                                            file: file.clone(),
                                            ident: obj.identifier.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.constants.len() > 0 {
                            config = config.push(text(format!("Consts: {}", dict.constants.len())));
                            for obj in &dict.constants {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.identifier)).on_press(
                                        Message::Page(States::Error {
                                            file: file.clone(),
                                            ident: obj.identifier.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                    }
                    FileType::Dll => {
                        let dict = self.objects.1.get(&file.name).unwrap();
                        if dict.user_data.len() > 0 {
                            config =
                                config.push(text(format!("Userdata: {}", dict.user_data.len())));
                            for obj in &dict.user_data {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.name)).on_press(
                                        Message::Page(States::UserData {
                                            file: file.clone(),
                                            ident: obj.name.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.functions.len() > 0 {
                            config =
                                config.push(text(format!("Functions: {}", dict.functions.len())));
                            for obj in &dict.functions {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.name)).on_press(
                                        Message::Page(States::Function {
                                            file: file.clone(),
                                            block: None,
                                            ident: obj.name.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.enums.len() > 0 {
                            config = config.push(text(format!("Enums: {}", dict.enums.len())));
                            for obj in &dict.enums {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.name)).on_press(
                                        Message::Page(States::Enum {
                                            file: file.clone(),
                                            ident: obj.name.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        if dict.traits.len() > 0 {
                            config = config.push(text(format!("Traits: {}", dict.traits.len())));
                            for obj in &dict.traits {
                                let mut row = Row::new().spacing(10).push(
                                    Button::new(text::Text::new(&obj.name)).on_press(
                                        Message::Page(States::Trait {
                                            file: file.clone(),
                                            ident: obj.name.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }
                        /*if dict.consts.len() > 0 {
                            config = config.push(text(format!("Consts: {}", dict.consts.len())));
                            for obj in &dict.consts {
                                let mut row = Row::new().spacing(10)
                                    .push(
                                    Button::new(text::Text::new(&obj.name)).on_press(
                                        Message::Page(States::Const {
                                            file: file.clone(),
                                            ident: obj.name.clone(),
                                        }),
                                    ),
                                );
                                match &obj.docs {
                                    Some(docs) => {
                                        row = row.push(text(docs.lines().nth(0).unwrap()));
                                    }
                                    None => {}
                                }
                                config = config.push(row);
                            }
                        }*/
                    }
                };
                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Enum { file, ident } => {
                let (variants, docs) = match file.file_type {
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
                        let docs = match &enum_.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (config, docs)
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
                        let docs = match &enum_.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (config, docs)
                    }
                };
                let title = text(format!("Enum: {}", ident));
                let config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(title)
                    .push(docs)
                    .push(variants);
                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Struct { file, ident } => {
                let (fields, methods, docs) = match file.file_type {
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
                        let docs = match &struct_.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (fields, methods, docs)
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
                        let docs = match &struct_.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (fields, methods, docs)
                    }
                };
                let mut config = Column::new().spacing(10).push(navigation);

                config = config.push(text(format!("Struct: {}", ident)));
                config = config.push(docs);
                config = config.push(fields);
                config = config.push(methods);

                let scrollable = scrollable::Scrollable::new(config)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill);
                scrollable.into()
            }
            States::Function { file, block, ident } => {
                let mut kind = String::from("Function");
                let (params, returns, docs) = match file.file_type {
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
                        let returns = match &fun.return_type {
                            Some(return_type) => text(format!("Returns: {:?}", return_type)),
                            None => text("Returns: *void*"),
                        };
                        let docs = match &fun.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (params, returns, docs)
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
                        let docs = match &fun.docs {
                            Some(docs) => text(docs.clone()),
                            None => text(""),
                        };
                        (params, returns, docs)
                    }
                };
                let mut config = Column::new()
                    .spacing(10)
                    .push(navigation)
                    .push(text(format!("{}: {}", kind, ident)));

                config = config.push(docs);
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
                    let row = Row::new().spacing(10).push(
                        Button::new(text::Text::new(method.identifier.as_ref().unwrap())).on_press(
                            Message::Page(States::Function {
                                file: file.clone(),
                                block: None,
                                ident: method.identifier.clone().unwrap(),
                            }),
                        ),
                    );
                    match &method.docs {
                        Some(docs) => {
                            config = config.push(row.push(text(docs.lines().nth(0).unwrap())));
                        }
                        None => {
                            config = config.push(row);
                        }
                    }
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
                    let row = Row::new().spacing(10).push(
                        Button::new(text::Text::new(&method.name)).on_press(Message::Page(
                            States::Function {
                                file: file.clone(),
                                block: Some(ident.clone()),
                                ident: method.name.clone(),
                            },
                        )),
                    );
                    match &method.docs {
                        Some(docs) => {
                            methods = methods.push(row.push(text(docs.lines().nth(0).unwrap())));
                        }
                        None => {
                            methods = methods.push(row);
                        }
                    }
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

pub mod bin_lens {

    use std::vec;

    use iced::{
        widget::{TextInput},
    };
    use runtime::runtime_types::{Types, PointerTypes, Instructions};
    use stringify::Data;

    use super::*;

    #[derive(Debug, Clone)]
    pub enum States {
        Main,
        Heap,
        Stack,
        Strings,
        Libs,
        EntryPoint,
        Instructions,
        NonPrimitives,
    }

    #[derive(Debug, Clone)]
    enum Navigation {
        Back,
        Forward,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Navigation(Navigation),
        Page(States),
        Search(String),
        SearchSubmit,
    }

    pub struct LensInstructions {
        pub instructions: HashMap<String, Vec<Instructions>>,
        pub entry_point: (String, usize),
    }

    pub struct BinLens {
        state: States,
        history: (usize, Vec<States>),
        data: Data,
        project_name: String,
        search: String,
        instructions: LensInstructions,
    }

    impl BinLens {
        pub fn back(&mut self) {
            if self.history.0 == 0 {
                return;
            }
            self.history.0 -= 1;
            self.state = self.history.1[self.history.0].clone();
        }

        pub fn forward(&mut self) {
            if self.history.0 + 1 >= self.history.1.len() {
                return;
            }
            self.history.0 += 1;
            self.state = self.history.1[self.history.0].clone();
        }

        /// Returns true if the object should be displayed.
        ///
        /// Each search is ranked based on the following criteria:
        /// 1. Search is empty: full points
        /// 2. The object's name equals the search (case sensitive): 1/1 points
        /// 3. The object's name equals the search (case insensitive): 1/2 points
        /// 4. The object's name contains the search (case sensitive): 1/3 points
        /// 5. The object's name contains the search (case insensitive): 1/4 points
        ///
        /// If none of the above are true, the object will be ranked based on the following criteria:
        /// Characters in the search that are in the same order as in the object's name will be counted where each consecutive match will be worth the last match's points + match worth.
        /// Match worth is determined by whether the match is case sensitive or not. (case sensitive: 1, case insensitive: 1/2)
        ///
        pub fn searched(&self, search: &str) -> Option<i32> {
            if self.search.len() == 0 {
                return Some(i32::MAX);
            }
            if self.search == search {
                return Some(i32::MAX);
            }
            if search.to_lowercase() == self.search.to_lowercase() {
                return Some(i32::MAX / 2);
            }
            if search.contains(&self.search) {
                return Some(i32::MAX / 3);
            }
            if search.to_lowercase().contains(&self.search.to_lowercase()) {
                return Some(i32::MAX / 4);
            }

            let mut points = 0;
            let mut last_match = 0;
            let mut search_chars = self.search.chars();
            for char in search.chars() {
                if let Some(search_char) = search_chars.next() {
                    if char == search_char {
                        points += last_match + 2;
                        last_match += 2;
                    } else if char.to_lowercase().next() == search_char.to_lowercase().next() {
                        points += last_match + 1;
                        last_match += 1;
                    } else {
                        last_match = 0;
                    }
                } else {
                    break;
                }
            }
            if points == 0 {
                return None;
            }
            Some(points)
        }

        fn print_value(&self, value: Types) -> String {
            match &value {
                Types::Bool(val) => format!("bool: {}", val),
                Types::Int(val) => format!("int: {}", val),
                Types::Float(val) => format!("float: {}", val),
                Types::Uint(val) => format!("uint: {}", val),
                Types::Char(val) => format!("char: {}", val),
                Types::Function(val) => format!("function: {}", val), // todo: print function
                Types::Null => format!("null"),
                Types::Pointer(loc, kind) => {
                    let mut res = String::from(format!("Pointer({loc}, {kind}): "));
                    match kind {
                        PointerTypes::Char(c) => {
                            res.push_str(format!("{}", c).as_str());
                        }
                        PointerTypes::Stack => {
                            res.push_str(&format!("{}", self.print_value(self.data.values[*loc])));
                        }
                        PointerTypes::Object => {
                            res.push_str(&format!("{}", self.print_value(self.data.heap[*loc][0])));
                        }
                        PointerTypes::Heap(pos) => {
                            res.push_str(&format!("{}", self.print_value(self.data.heap[*loc][*pos])));
                        }
                        PointerTypes::String => {
                            res.push_str(&format!("{}", self.data.strings[*loc]));
                        }
                        PointerTypes::UserData => res.push_str("UserData"),
                    }
                    res
                }
                Types::NonPrimitive(idx) => {
                    format!("NonPrimitive({}): {}", idx, self.data.non_primitives[*idx].name)
                }
                Types::Void => format!("*void*"),
            }
        }
    }

    impl Application for BinLens {
        fn new(flags: BinLensFlags) -> (BinLens, iced::Command<Message>) {
            let mut instructions = HashMap::new();
            match &flags.objects.debug {
                Some(debug) => {
                    println!("{:?}", debug.lines);
                    if debug.lines.len() == 0 {
                        let mut all = Vec::new();
                        for instruction in flags.objects.instructions.iter() {
                            all.push(instruction.clone());
                        }
                        instructions.insert(String::from("main"), all);
                    }
                    let mut idx = 0;
                    for line in &debug.lines {
                        let mut all = Vec::new();
                        for i in idx..line.pos {
                            all.push(flags.objects.instructions[i].clone());
                        }
                        idx = line.pos;
                        instructions.insert(debug.labels[line.label.unwrap()].msg.clone(), all);
                    }
                }
                None => {
                    let mut all = Vec::new();
                    for instruction in flags.objects.instructions.iter() {
                        all.push(instruction.clone());
                    }
                    instructions.insert(String::from("main"), all);
                }
            }
            (
                Self {
                    state: States::Main,
                    history: (0, Vec::new()),
                    data: flags.objects,
                    project_name: flags.project_name,
                    search: String::new(),
                    instructions: LensInstructions {
                        instructions,
                        entry_point: (String::new(), 0),
                    },
                },
                Command::none(),
            )
        }

        fn title(&self) -> String {
            format!("Ruda Lens - {}", self.project_name)
        }

        fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
            match &message {
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
                Message::Page(page) => {
                    self.history.1.truncate(self.history.0);
                    self.history.0 += 1;
                    self.history.1.push(self.state.clone());
                    self.state = page.clone();
                    iced::Command::none()
                }
                Message::Search(txt) => {
                    self.search = txt.clone();
                    iced::Command::none()
                }
                Message::SearchSubmit => iced::Command::none(),
            }
        }

        fn theme(&self) -> iced::Theme {
            iced::Theme::Dark
        }

        fn style(&self) -> iced::theme::Application {
            iced::theme::Application::default()
        }

        fn view(&self) -> Element<'_, Self::Message> {
            let config = Column::new().spacing(10).push(
                TextInput::new("Search", &self.search)
                    .on_input(Message::Search)
                    .on_submit(Message::SearchSubmit)
                    .width(iced::Length::Fixed(350.)),
            );
            let navigation = Row::new()
                .push(
                    Button::new(text::Text::new("Back"))
                        .on_press(Message::Navigation(Navigation::Back)),
                )
                .push(
                    Button::new(text::Text::new("Forward"))
                        .on_press(Message::Navigation(Navigation::Forward)),
                );

            let mut config = config.push(container::Container::new(navigation));
            match &self.state {
                States::Main => {
                    // table of contents
                    config = config.push(text("Table of contents:"));
                    let heap =
                        Button::new(text::Text::new("Heap")).on_press(Message::Page(States::Heap));
                    let stack = Button::new(text::Text::new("Constants"))
                        .on_press(Message::Page(States::Stack));
                    let strings = Button::new(text::Text::new("Strings"))
                        .on_press(Message::Page(States::Strings));
                    let libs =
                        Button::new(text::Text::new("Libs")).on_press(Message::Page(States::Libs));
                    let entry_point = Button::new(text::Text::new("Entry point"))
                        .on_press(Message::Page(States::EntryPoint));
                    let instructions = Button::new(text::Text::new("Instructions"))
                        .on_press(Message::Page(States::Instructions));
                    let non_primitives = Button::new(text::Text::new("Non primitives"))
                        .on_press(Message::Page(States::NonPrimitives));
                    config = config
                        .push(heap)
                        .push(stack)
                        .push(strings)
                        .push(libs)
                        .push(entry_point)
                        .push(instructions)
                        .push(non_primitives);
                }
                States::Strings => {
                    // idx. content
                    let mut strings = Vec::new();
                    for string in self.data.strings.iter().enumerate() {
                        let rank = match self.searched(string.1) {
                            Some(rank) => rank,
                            None => continue,
                        };
                        strings.push((string.0, rank));
                    }
                    strings.sort_by(|a, b| b.1.cmp(&a.1));
                    let mut temp = Column::new().spacing(10);
                    for string in strings {
                        temp = temp.push(text(format!(
                            "{}: {}",
                            string.0, self.data.strings[string.0]
                        )));
                    }
                    config = config.push(temp);
                    config = config
                        .push(text(format!("Total count: {}", self.data.strings.len())))
                        .push(text(format!(
                            "Total size: {}",
                            self.data
                                .strings
                                .iter()
                                .fold(0, |acc, string| acc + string.len())
                        )));
                }
                States::Stack => {
                    // idx. value
                    let mut stack = Vec::new();
                    for value in self.data.values.iter().enumerate() {
                        let rank = match self.searched(&self.print_value(value.1.clone())) {
                            Some(rank) => rank,
                            None => continue,
                        };
                        stack.push((value.0, rank));
                    }
                    stack.sort_by(|a, b| b.1.cmp(&a.1));
                    let mut temp = Column::new();
                    for value in stack {
                        temp = temp.push(text(format!(
                            "{}: {}",
                            value.0,
                            self.print_value(self.data.values[value.0].clone())
                        ))).width(iced::Length::Fill);
                    }
                    config = config.push(temp);
                    config = config.push(text(format!(
                        "Total count: {}",
                        self.data.values.len()
                    )));
                },
                States::Heap => todo!(),
                States::Libs => todo!(),
                States::EntryPoint => todo!(),
                States::Instructions => {
                    let mut instructions = Vec::new();
                    for instruction in self.instructions.instructions.iter() {
                        let rank = match self.searched(instruction.0) {
                            Some(rank) => rank,
                            None => continue,
                        };
                        instructions.push((instruction.0.clone(), rank));
                    }
                    instructions.sort_by(|a, b| b.1.cmp(&a.1));
                    let mut temp = Column::new().spacing(10);
                    for instruction in instructions {
                        temp = temp.push(text(format!(
                            "{}: {}",
                            instruction.0,
                            self.instructions.instructions[&instruction.0]
                                .iter()
                                .fold(String::new(), |acc, instruction| {
                                    format!("{}\n{}", acc, instruction)
                                })
                        )).width(iced::Length::Fill));
                    }
                    config = config.push(temp);
                    config = config.push(text(format!(
                        "Total count: {}",
                        self.instructions.instructions.len()
                    )));
                }
                States::NonPrimitives => {
                    let mut non_primitives = Vec::new();
                    for non_primitive in self.data.non_primitives.iter().enumerate() {
                        let rank = match self.searched(&non_primitive.1.name) {
                            Some(rank) => rank,
                            None => continue,
                        };
                        non_primitives.push((non_primitive.0, rank));
                    }
                    non_primitives.sort_by(|a, b| b.1.cmp(&a.1));
                    let mut temp = Column::new().spacing(10);
                    for non_primitive in non_primitives {
                        temp = temp.push(text(format!(
                            "{}: {}",
                            non_primitive.0, self.data.non_primitives[non_primitive.0].name
                        )));
                        temp = temp.push(text(format!(
                            "\t- Size: {}, Kind: {}, Ptrs: {}",
                            self.data.non_primitives[non_primitive.0].len,
                            self.data.non_primitives[non_primitive.0].kind,
                            self.data.non_primitives[non_primitive.0].pointers,
                        )));
                    }
                    config = config.push(temp);
                    config = config.push(text(format!(
                        "Total count: {}",
                        self.data.non_primitives.len()
                    )));
                }
            }
            let scrollable = scrollable::Scrollable::new(config)
                .height(iced::Length::Fill)
                .width(iced::Length::Fill);
            container::Container::new(scrollable).padding(10).into()
        }

        type Executor = executor::Default;

        type Message = Message;

        type Theme = iced::Theme;

        type Flags = BinLensFlags;
    }

    pub struct BinLensFlags {
        pub objects: Data,
        pub project_name: String,
    }
}
