//! Chui Macros
//!
//! Thanks to this guide: <https://github.com/imbolc/rust-derive-macro-guide> (2023-05-24)

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(trainer))]
/// Options for use in macros.
struct Opts {
    /// Base option.
    base: Option<bool>,
}

/// Derive Trainer trait.
///
/// # Panics
///
/// Not sure where this can panic.
///
/// TODO: Find out where this can panic.
#[proc_macro_derive(Trainer, attributes(trainer))]
pub fn derive_trainer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let base = match opts.base {
        Some(do_base) => {
            if !do_base {
                let output = quote! {};
                return output.into();
            }

            quote! {
                /// Create a new `Trainer` object.
                fn new() -> #ident {
                    Self::default()
                }

                /// Run the main training simulator. It doesn't matter if we're only training Numeric,
                /// Alpha, or Both, we use this loop for all 3 session types.
                ///
                /// Loop until the user quits the session. Until quit, this loop generates a problem,
                /// a user input is expected, and the answer checked for correctness. When the user
                /// quits, print a score sheet and go back to the main state of the application
                /// (`CommandType::Help`).
                fn train(&mut self, command_type: CommandType) {
                    self.command_type = command_type;
                    self.session_timer = SystemTime::now();

                    println!(
                        " = Train Yourself in {} {} =",
                        self.get_name_verbose(),
                        self.get_names_verbose(0, true)
                    );
                    println!("");

                    loop {
                        self.generate_problem();
                        self.get_input();
                        // We do not store the command type when calling `self.process_command()` because
                        // we don't want to change the state of the application right before checking
                        // user input.
                        match self.process_command(false) {
                            CommandType::Input => self.solve_answer(),
                            CommandType::Help => self.print_help(),
                            CommandType::Quit => {
                                self.quit();
                                break;
                            }
                            _ => continue,
                        }
                    }
                }

                /// Get the verbose name of the session.
                fn get_name_verbose(&self) -> String {
                    self.name_verbose.clone()
                }

                /// Get the Vector of verbose names of the session.
                fn get_names_verbose(&self, idx: usize, debug: bool) -> String {
                    if debug {
                        return format!("{:?}", self.names_verbose);
                    }

                    if let Some(name) = self.names_verbose.get(idx) {
                        name.clone()
                    } else {
                        String::new()
                    }
                }

                /// Return a String representing the `self.get_help()` `?` and `help` text.
                fn get_help_msg_string(&self) -> String {
                    if self.command_type == CommandType::Help {
                        "* ?, or help".to_string()
                    } else {
                        "  ?, or help".to_string()
                    }
                }

                /// Print the output correlating to a correct answer.
                fn print_correct(&self) {
                    println!(
                        " +++ Correct! ({} correct, {} incorrect)",
                        self.vec_correct.len(),
                        self.vec_incorrect.len()
                    );
                }

                /// Print the output correlating to a correct answer.
                fn print_incorrect(&self) {
                    println!(
                        " --- Incorrect! ({} correct, {} incorrect)",
                        self.vec_correct.len(),
                        self.vec_incorrect.len()
                    );
                }

                /// Print the final scores and reset the Coordinate Trainer to the default run state.
                ///
                /// Note: This doesn't actually quit the application.
                ///
                /// TODO: Maybe it should?
                fn quit(&mut self) {
                    self.print_scores();
                    *self = Self::new();
                }

                /// Given user input on `self.input`, process the command that follows that input. If
                /// `set` is true, set `self.command_type` as the processed command, otherwise just
                /// return that variant.
                ///
                /// Note: `CommandType` is Copy and Clone.
                fn process_command(&mut self, set: bool) -> CommandType {
                    let command_type = if self.input.eq("?") || self.input.eq("help") {
                        CommandType::Help
                    } else if self.input.eq("q") || self.input.eq("quit") || self.input.eq("exit") {
                        CommandType::Quit
                    } else {
                        CommandType::Input
                    };

                    if set {
                        self.command_type = command_type;
                    }

                    command_type
                }

                fn generate_problem(&mut self) {}
                fn evaluate_answer(&mut self) {}
                fn solve_answer(&mut self) {}
            }
        }
        None => quote! {},
    };

    let output = quote! {
        impl Trainer for #ident {
            #base
        }
    };
    output.into()
}

/// Derive Coordinate trait.
#[proc_macro_derive(Coordinate)]
pub fn derive_coordinate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput { ident, .. } = input;

    let primitive_types = [
        quote! { u8 },
        quote! { u16 },
        quote! { u32 },
        quote! { u64 },
        quote! { u128 },
        quote! { usize },
        quote! { i8 },
        quote! { i16 },
        quote! { i32 },
        quote! { i64 },
        quote! { i128 },
        quote! { isize },
    ];

    let primitive_type_refs = [
        quote! { &u8 },
        quote! { &u16 },
        quote! { &u32 },
        quote! { &u64 },
        quote! { &u128 },
        quote! { &usize },
        quote! { &i8 },
        quote! { &i16 },
        quote! { &i32 },
        quote! { &i64 },
        quote! { &i128 },
        quote! { &isize },
    ];

    let mut output = quote! {};

    //
    // TryFrom impls
    //

    for t in primitive_types.iter() {
        for u in primitive_types.iter() {
            output = quote! {
                #output

                impl TryFrom<(#t, #u)> for #ident {
                    type Error = ChuiError;

                    fn try_from(coord: (#t, #u)) -> ChuiResult<Coord> {
                        if let Ok(file) = u8::try_from(coord.0) {
                            if let Ok(rank) = u8::try_from(coord.1) {
                                if let Ok(file) = NonMaxU8::try_from(file) {
                                    if let Ok(rank) = NonMaxU8::try_from(rank) {
                                        Ok(Coord { file, rank })
                                    } else {
                                        Err(ChuiError::InvalidRank(format!(
                                            "{} is an invalid rank",
                                            rank
                                        )))
                                    }
                                } else {
                                    Err(ChuiError::InvalidFile(format!(
                                        "{} is an invalid file",
                                        file
                                    )))
                                }
                            } else {
                                Err(ChuiError::InvalidTypeConversion(format!(
                                    "{} could not be converted to a valid u8 type",
                                    coord.1
                                )))
                            }
                        } else {
                            Err(ChuiError::InvalidTypeConversion(format!(
                                "{} could not be converted to a valid u8 type",
                                coord.0
                            )))
                        }
                    }
                }

                impl PartialEq<(#t, #u)> for #ident {
                    fn eq(&self, coord: &(#t, #u)) -> bool {
                        if let Ok(new_coord) = Coord::try_from(*coord) {
                            *self == new_coord
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    for t in primitive_types.iter() {
        for u in primitive_type_refs.iter() {
            output = quote! {
                #output

                impl TryFrom<(#t, #u)> for #ident {
                    type Error = ChuiError;

                    fn try_from(coord: (#t, #u)) -> ChuiResult<Coord> {
                        if let Ok(file) = u8::try_from(coord.0) {
                            if let Ok(rank) = u8::try_from(*coord.1) {
                                if let Ok(file) = NonMaxU8::try_from(file) {
                                    if let Ok(rank) = NonMaxU8::try_from(rank) {
                                        Ok(Coord { file, rank })
                                    } else {
                                        Err(ChuiError::InvalidRank(format!(
                                            "{} is an invalid rank",
                                            rank
                                        )))
                                    }
                                } else {
                                    Err(ChuiError::InvalidFile(format!(
                                        "{} is an invalid file",
                                        file
                                    )))
                                }
                            } else {
                                Err(ChuiError::InvalidTypeConversion(format!(
                                    "{} could not be converted to a valid u8 type",
                                    coord.1
                                )))
                            }
                        } else {
                            Err(ChuiError::InvalidTypeConversion(format!(
                                "{} could not be converted to a valid u8 type",
                                coord.0
                            )))
                        }
                    }
                }

                impl PartialEq<(#t, #u)> for #ident {
                    fn eq(&self, coord: &(#t, #u)) -> bool {
                        if let Ok(new_coord) = Coord::try_from(*coord) {
                            *self == new_coord
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    for t in primitive_type_refs.iter() {
        for u in primitive_types.iter() {
            output = quote! {
                #output

                impl TryFrom<(#t, #u)> for #ident {
                    type Error = ChuiError;

                    fn try_from(coord: (#t, #u)) -> ChuiResult<Coord> {
                        if let Ok(file) = u8::try_from(*coord.0) {
                            if let Ok(rank) = u8::try_from(coord.1) {
                                if let Ok(file) = NonMaxU8::try_from(file) {
                                    if let Ok(rank) = NonMaxU8::try_from(rank) {
                                        Ok(Coord { file, rank })
                                    } else {
                                        Err(ChuiError::InvalidRank(format!(
                                            "{} is an invalid rank",
                                            rank
                                        )))
                                    }
                                } else {
                                    Err(ChuiError::InvalidFile(format!(
                                        "{} is an invalid file",
                                        file
                                    )))
                                }
                            } else {
                                Err(ChuiError::InvalidTypeConversion(format!(
                                    "{} could not be converted to a valid u8 type",
                                    coord.1
                                )))
                            }
                        } else {
                            Err(ChuiError::InvalidTypeConversion(format!(
                                "{} could not be converted to a valid u8 type",
                                coord.0
                            )))
                        }
                    }
                }

                impl PartialEq<(#t, #u)> for #ident {
                    fn eq(&self, coord: &(#t, #u)) -> bool {
                        if let Ok(new_coord) = Coord::try_from(*coord) {
                            *self == new_coord
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    for t in primitive_type_refs.iter() {
        for u in primitive_type_refs.iter() {
            output = quote! {
                #output

                impl TryFrom<(#t, #u)> for #ident {
                    type Error = ChuiError;

                    fn try_from(coord: (#t, #u)) -> ChuiResult<Coord> {
                        if let Ok(file) = u8::try_from(*coord.0) {
                            if let Ok(rank) = u8::try_from(*coord.1) {
                                if let Ok(file) = NonMaxU8::try_from(file) {
                                    if let Ok(rank) = NonMaxU8::try_from(rank) {
                                        Ok(Coord { file, rank })
                                    } else {
                                        Err(ChuiError::InvalidRank(format!(
                                            "{} is an invalid rank",
                                            rank
                                        )))
                                    }
                                } else {
                                    Err(ChuiError::InvalidFile(format!(
                                        "{} is an invalid file",
                                        file
                                    )))
                                }
                            } else {
                                Err(ChuiError::InvalidTypeConversion(format!(
                                    "{} could not be converted to a valid u8 type",
                                    coord.1
                                )))
                            }
                        } else {
                            Err(ChuiError::InvalidTypeConversion(format!(
                                "{} could not be converted to a valid u8 type",
                                coord.0
                            )))
                        }
                    }
                }

                impl PartialEq<(#t, #u)> for #ident {
                    fn eq(&self, coord: &(#t, #u)) -> bool {
                        if let Ok(new_coord) = Coord::try_from(*coord) {
                            *self == new_coord
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    output.into()
}
