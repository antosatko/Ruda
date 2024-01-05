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
    use std::fmt::format;

    use stringify::Data;

    use super::*;

    #[derive(Debug, Clone)]
    pub enum States {
        Main,
        Search(String),
    }

    #[derive(Debug, Clone)]
    enum Navigation {
        Back,
        Forward,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Navigation(Navigation),
    }

    pub struct BinLens {
        state: States,
        history: (usize, Vec<States>),
        data: Data,
        project_name: String,
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
    }

    impl Application for BinLens {
        fn new(flags: BinLensFlags) -> (BinLens, iced::Command<Message>) {
            (
                Self {
                    state: States::Main,
                    history: (0, Vec::new()),
                    data: flags.objects,
                    project_name: flags.project_name,
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
            }
        }

        fn theme(&self) -> iced::Theme {
            iced::Theme::Dark
        }

        fn style(&self) -> iced::theme::Application {
            iced::theme::Application::default()
        }

        fn view(&self) -> Element<'_, Self::Message> {
            return text::Text::new("Hello world!").into();
            /*let mut navigation = Row::new()
                .push(
                    Button::new(text::Text::new("Back"))
                        .on_press(Message::Navigation(Navigation::Back)),
                )
                .push(
                    Button::new(text::Text::new("Forward"))
                        .on_press(Message::Navigation(Navigation::Forward)),
                );
            match self.state {
                States::Main => todo!(),
                States::Search(_) => todo!(),
            }*/
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
