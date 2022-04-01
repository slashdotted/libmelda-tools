// Melda Tools - Tools for Melda: Delta State JSON CRDT
// Copyright (C) 2022 Amos Brocco <amos.brocco@supsi.ch>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not,ls see <http://www.gnu.org/licenses/>.
use std::sync::{Arc, RwLock};

use clap::{Parser, Subcommand};
use melda::{
    adapter::Adapter, filesystemadapter::FilesystemAdapter, melda::Melda, solidadapter::SolidAdapter,
};
use serde_json::{Value};
use url::Url;

#[derive(Debug, Parser)]
#[clap(name = "libmelda-tools")]
#[clap(about = "CLI tool for libmelda", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Adds files to myapp
    
    #[clap(arg_required_else_help = true)]
    Update { 
        #[clap(required = true,short, long)]
        target: Option<String>,

        #[clap(required = true, short, long)]
        jsonfile: Option<String>,

        #[clap(short, long)]
        author: Option<String>,

        #[clap(short, long)]
        description: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>
    },
    #[clap(arg_required_else_help = true)]
    Read {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>
    },
    #[clap(arg_required_else_help = true)]
    Meld {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(long)]
        susername: Option<String>,

        #[clap(long)]
        spassword: Option<String>,

        #[clap(required = true, short, long)]
        target: Option<String>,

        #[clap(long)]
        tusername: Option<String>,

        #[clap(long)]
        tpassword: Option<String>,
    },
    #[clap(arg_required_else_help = true)]
    Debug {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>
    },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Update { target, jsonfile, author, description , username, password} => {
            match Url::parse(&target.unwrap()) {
                Ok(url) => {
                    let adapter: Box<dyn Adapter> = if url.scheme().eq("file") {
                        Box::new(FilesystemAdapter::new(url.path()).expect("cannot_initialize_adapter"))
                    } else if url.scheme().eq("solid") {
                        Box::new(
                            SolidAdapter::new(
                                "https://".to_string() + &url.host().unwrap().to_string(),
                                url.path().to_string() + "/", username, password)
                            .expect("cannot_initialize_adapter"),
                        )
                    } else {
                        panic!("invalid_adapter");
                    };
                    let mut m = Melda::new(Arc::new(RwLock::new(adapter))).unwrap();
                    let contents = std::fs::read_to_string(jsonfile.unwrap()).expect("Failed to read JSON file");
                    let v: Value = serde_json::from_str(&contents).expect("Not a JSON value");
                    let o = v.as_object().expect("Not an object");
                    m.update(o.clone()).expect("Failed to update");
                    let mut i = serde_json::Map::<String,Value>::new();
                    if author.is_some() {
                        i.insert("author".to_string(), Value::from(author.unwrap()));
                    }
                    if description.is_some() {
                        i.insert("description".to_string(), Value::from(description.unwrap()));
                    }
                    let blockid = if i.is_empty() {
                        m.commit(None, false)
                    } else {
                        m.commit(Some(i), false)
                    }.expect("Failed to commit");
                    if blockid.is_some() {
                        println!("Committed block {}", blockid.unwrap());
                    } else {
                        println!("Nothing to commit");
                    }
                },
                _ => {
                    eprintln!("Invalid Url");
                }
            }
        },
        Commands::Read { source, username, password} => {
            match Url::parse(&source.unwrap()) {
                Ok(url) => {
                    let adapter: Box<dyn Adapter> = if url.scheme().eq("file") {
                        Box::new(FilesystemAdapter::new(url.path()).expect("cannot_initialize_adapter"))
                    } else if url.scheme().eq("solid") {
                        Box::new(
                            SolidAdapter::new(
                                "https://".to_string() + &url.host().unwrap().to_string(),
                                url.path().to_string() + "/", username, password)
                            .expect("cannot_initialize_adapter"),
                        )
                    } else {
                        panic!("invalid_adapter");
                    };
                    let m = Melda::new(Arc::new(RwLock::new(adapter))).unwrap();
                    let data = m.read().expect("Failed to read");
                    let content = serde_json::to_string(&data).unwrap();
                    println!("{}", content);
                },
                _ => {
                    eprintln!("Invalid Url");
                }
            }
        },
        Commands::Debug { source, username, password} => {
            match Url::parse(&source.unwrap()) {
                Ok(url) => {
                    let adapter: Box<dyn Adapter> = if url.scheme().eq("file") {
                        Box::new(FilesystemAdapter::new(url.path()).expect("cannot_initialize_adapter"))
                    } else if url.scheme().eq("solid") {
                        Box::new(
                            SolidAdapter::new(
                                "https://".to_string() + &url.host().unwrap().to_string(),
                                url.path().to_string() + "/", username, password)
                            .expect("cannot_initialize_adapter"),
                        )
                    } else {
                        panic!("invalid_adapter");
                    };
                    let m = Melda::new(Arc::new(RwLock::new(adapter))).unwrap();
                    m.debug();
                },
                _ => {
                    eprintln!("Invalid Url");
                }
            }
        },
        Commands::Meld { source, target , susername, spassword, tusername, tpassword} => {
            match Url::parse(&source.unwrap()) {
                Ok(surl) => {
                    match Url::parse(&target.unwrap()) {
                        Ok(turl) => {
                            let sadapter: Box<dyn Adapter> = if surl.scheme().eq("file") {
                                Box::new(FilesystemAdapter::new(surl.path()).expect("cannot_initialize_adapter"))
                            } else if surl.scheme().eq("solid") {
                                Box::new(
                                    SolidAdapter::new(
                                        "https://".to_string() + &surl.host().unwrap().to_string(),
                                        surl.path().to_string() + "/", susername, spassword)
                                    .expect("Cannot initialize source adapter"),
                                )
                            } else {
                                panic!("Invalid source adapter");
                            };

                            let tadapter: Box<dyn Adapter> = if turl.scheme().eq("file") {
                                Box::new(FilesystemAdapter::new(turl.path()).expect("cannot_initialize_adapter"))
                            } else if turl.scheme().eq("solid") {
                                Box::new(
                                    SolidAdapter::new(
                                        "https://".to_string() + &turl.host().unwrap().to_string(),
                                        turl.path().to_string() + "/", tusername, tpassword)
                                    .expect("Cannot initialize target adapter"),
                                )
                            } else {
                                panic!("Invalid target adapter");
                            };

                            let s = Melda::new(Arc::new(RwLock::new(sadapter))).unwrap();
                            let mut t = Melda::new(Arc::new(RwLock::new(tadapter))).unwrap();
                            println!("{:?}", t.meld(&s).expect("Failed to meld"));
                        },
                        _ => {
                            eprintln!("Invalid target Url");
                        }
                    }
                },
                _ => {
                    eprintln!("Invalid source Url");
                }
            }
        },
    }
    
}