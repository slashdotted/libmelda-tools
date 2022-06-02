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
use std::{
    collections::VecDeque,
    process::exit,
    sync::{Arc, RwLock},
};

use clap::{Parser, Subcommand};
use melda::{
    adapter::Adapter, filesystemadapter::FilesystemAdapter, flate2adapter::Flate2Adapter,
    melda::Melda, solidadapter::SolidAdapter,
};
use serde_json::Value;
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
    /// Updates a Melda CRDT
    #[clap(arg_required_else_help = true)]
    Update {
        #[clap(required = true, short, long)]
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
        password: Option<String>,
    },
    /// Reads a Melda CRDT
    #[clap(arg_required_else_help = true)]
    Read {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,

        #[clap(short, long)]
        block: Option<String>,
    },
    /// Melds two Melda CRDTs
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
    /// Show the log of a Melda CRDT
    #[clap(arg_required_else_help = true)]
    Log {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,

        #[clap(short, long)]
        block: Option<String>,
    },
    /// Show which objects are in conflict
    #[clap(arg_required_else_help = true)]
    Conflicts {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,
    },
    /// Resolves a conflict by picking up a winner
    #[clap(arg_required_else_help = true)]
    Resolve {
        #[clap(required = true, short, long)]
        target: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,

        #[clap(short, long)]
        object: Option<String>,

        #[clap(short, long)]
        winner: Option<String>,
    },
    /// Prints the history of revisions of an object
    #[clap(arg_required_else_help = true)]
    History {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,

        #[clap(short, long)]
        object: String,
    },
    /// Prints the value of an object for the given revision (or the winner)
    #[clap(arg_required_else_help = true)]
    Value {
        #[clap(required = true, short, long)]
        source: Option<String>,

        #[clap(short, long)]
        username: Option<String>,

        #[clap(short, long)]
        password: Option<String>,

        #[clap(short, long)]
        object: String,

        #[clap(short, long)]
        revision: Option<String>,
    },
}

fn print_block_detail(m: &Melda, block_id: &str, is_anchor: bool) -> Option<Vec<String>> {
    if let Some(block) = m.get_block(block_id).expect("Failed to get block") {
        if is_anchor {
            println!("(A) Block: {}", block_id);
        } else if block.parents.is_none() {
            println!("(O) Block: {}", block_id);
        } else {
            println!("(-) Block: {}", block_id);
        }
        if block.info.is_some() {
            println!(
                "\t\tInformation: {}",
                serde_json::to_string(&Value::from(block.info.unwrap())).unwrap()
            )
        }
        if block.packs.is_some() {
            println!("\t\tPacks: {:?}", block.packs.unwrap());
        }
        if block.parents.is_some() {
            let parents = block.parents.unwrap();
            println!("\t\tParents: {:?}", parents);
            Some(parents)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_adapter(url: Url, username: Option<String>, password: Option<String>) -> Box<dyn Adapter> {
    if url.scheme().eq("file") {
        Box::new(FilesystemAdapter::new(url.path()).expect("cannot_initialize_adapter"))
    } else if url.scheme().eq("solid") {
        Box::new(
            SolidAdapter::new(
                "https://".to_string() + &url.host().unwrap().to_string(),
                url.path().to_string() + "/",
                username,
                password,
            )
            .expect("cannot_initialize_adapter"),
        )
    } else if url.scheme().eq("file+flate") {
        Box::new(Flate2Adapter::new(Arc::new(RwLock::new(Box::new(
            FilesystemAdapter::new(url.path()).expect("cannot_initialize_adapter"),
        )))))
    } else if url.scheme().eq("solid+flate") {
        Box::new(Flate2Adapter::new(Arc::new(RwLock::new(Box::new(
            SolidAdapter::new(
                "https://".to_string() + &url.host().unwrap().to_string(),
                url.path().to_string() + "/",
                username,
                password,
            )
            .expect("cannot_initialize_adapter"),
        )))))
    } else {
        panic!("invalid_adapter");
    }
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Update {
            target,
            jsonfile,
            author,
            description,
            username,
            password,
        } => match Url::parse(&target.unwrap()) {
            Ok(url) => {
                let adapter = get_adapter(url, username, password);
                let mut m =
                    Melda::new(Arc::new(RwLock::new(adapter))).expect("Failed to inizialize Melda");
                let contents =
                    std::fs::read_to_string(jsonfile.unwrap()).expect("Failed to read JSON file");
                let v: Value = serde_json::from_str(&contents).expect("Not a JSON value");
                let o = v.as_object().expect("Not an object");
                m.update(o.clone()).expect("Failed to update");
                let mut i = serde_json::Map::<String, Value>::new();
                if author.is_some() {
                    i.insert("author".to_string(), Value::from(author.unwrap()));
                }
                if description.is_some() {
                    i.insert("description".to_string(), Value::from(description.unwrap()));
                }
                let blockid = if i.is_empty() {
                    m.commit(None)
                } else {
                    m.commit(Some(i))
                }
                .expect("Failed to commit");
                if blockid.is_some() {
                    println!("Committed block {}", blockid.unwrap());
                } else {
                    println!("Nothing to commit");
                }
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
        Commands::Read {
            source,
            username,
            password,
            block,
        } => match Url::parse(&source.unwrap()) {
            Ok(url) => {
                let adapter = get_adapter(url, username, password);
                if block.is_some() {
                    let m =
                        Melda::new_until(Arc::new(RwLock::new(adapter)), block.unwrap().as_str())
                            .expect("Failed to inizialize Melda");
                    let data = m.read().expect("Failed to read");
                    let content = serde_json::to_string(&data).unwrap();
                    println!("{}", content);
                } else {
                    let m = Melda::new(Arc::new(RwLock::new(adapter)))
                        .expect("Failed to inizialize Melda");
                    let data = m.read().expect("Failed to read");
                    let content = serde_json::to_string(&data).unwrap();
                    println!("{}", content);
                }
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
        Commands::Meld {
            source,
            target,
            susername,
            spassword,
            tusername,
            tpassword,
        } => match Url::parse(&source.unwrap()) {
            Ok(surl) => match Url::parse(&target.unwrap()) {
                Ok(turl) => {
                    let sadapter = get_adapter(surl, susername, spassword);
                    let tadapter = get_adapter(turl, tusername, tpassword);
                    let s = Melda::new(Arc::new(RwLock::new(sadapter)))
                        .expect("Failed to inizialize source Melda");
                    let mut t = Melda::new(Arc::new(RwLock::new(tadapter)))
                        .expect("Failed to inizialize target Melda");
                    println!("{:?}", t.meld(&s).expect("Failed to meld"));
                }
                _ => {
                    eprintln!("Invalid target Url");
                }
            },
            _ => {
                eprintln!("Invalid source Url");
            }
        },
        Commands::Log {
            source,
            username,
            password,
            block,
        } => match Url::parse(&source.unwrap()) {
            Ok(url) => {
                let adapter = get_adapter(url, username, password);
                if block.is_some() {
                    let block = block.unwrap();
                    let m = Melda::new_until(Arc::new(RwLock::new(adapter)), &block)
                        .expect("Failed to inizialize Melda");
                    let anchors = m.get_anchors();
                    let mut to_visit = VecDeque::new();
                    to_visit.push_back(block);
                    while !to_visit.is_empty() {
                        let block = to_visit.pop_front().unwrap();
                        match print_block_detail(&m, &block, anchors.contains(&block)) {
                            Some(parents) => {
                                parents.into_iter().for_each(|p| to_visit.push_back(p));
                            }
                            None => {}
                        }
                    }
                } else {
                    let m = Melda::new(Arc::new(RwLock::new(adapter)))
                        .expect("Failed to inizialize Melda");
                    let anchors = m.get_anchors();
                    let mut to_visit = VecDeque::new();
                    for a in &anchors {
                        to_visit.push_back(a.clone());
                    }
                    while !to_visit.is_empty() {
                        let block = to_visit.pop_front().unwrap();
                        match print_block_detail(&m, &block, anchors.contains(&block)) {
                            Some(parents) => {
                                parents
                                    .into_iter()
                                    .for_each(|p| to_visit.push_back(p.clone()));
                            }
                            None => {}
                        }
                    }
                }
            }
            _ => {
                eprintln!("Invalid source Url");
            }
        },
        Commands::Conflicts {
            source,
            username,
            password,
        } => match Url::parse(&source.unwrap()) {
            Ok(url) => {
                let adapter = get_adapter(url, username, password);
                let m =
                    Melda::new(Arc::new(RwLock::new(adapter))).expect("Failed to inizialize Melda");
                let in_conflict = m.in_conflict();
                for uuid in &in_conflict {
                    println!("{}:", uuid);
                    let winning = m.get_winner(uuid).unwrap();
                    let winning_value = m.get_value(uuid, &winning).expect("cannot_get_value");
                    println!(
                        "\t🏆 {}: {}",
                        winning,
                        serde_json::to_string(&winning_value).unwrap()
                    );
                    for r in &m.get_conflicting(uuid).unwrap() {
                        let conflict_value = m.get_value(uuid, &r).expect("cannot_get_value");
                        println!(
                            "\t🗲 {}: {}",
                            r,
                            serde_json::to_string(&conflict_value).unwrap()
                        );
                    }
                }
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
        Commands::Resolve {
            target,
            username,
            password,
            object,
            winner,
        } => match Url::parse(&target.unwrap()) {
            Ok(url) => {
                // Resolve specific uuid
                let adapter = get_adapter(url, username, password);
                let mut m =
                    Melda::new(Arc::new(RwLock::new(adapter))).expect("Failed to inizialize Melda");
                if object.is_some() {
                    let in_conflict = m.in_conflict();
                    let uuid = object.unwrap();
                    if !in_conflict.contains(&uuid) {
                        eprintln!("{} has no conflicts", uuid);
                        exit(1);
                    }
                    if winner.is_some() {
                        let winner = winner.unwrap();
                        let winning = m.get_winner(&uuid).unwrap();
                        let conflicting = m.get_conflicting(&uuid).unwrap();
                        if !conflicting.contains(&winner) && (&winning != &winner) {
                            eprintln!("{} not a valid winner", winner);
                            exit(2);
                        } else {
                            match m.resolve_as(&uuid, &winner) {
                                Ok(w) => {
                                    println!("{} resolved as {} (previous: {})", uuid, w, winning)
                                }
                                Err(e) => {
                                    eprintln!(
                                        "{} failed to resolve as {}: {}",
                                        uuid,
                                        winner,
                                        e.to_string()
                                    );
                                    exit(3);
                                }
                            };
                        }
                    } else {
                        let winning = m.get_winner(&uuid).unwrap();
                        match m.resolve_as(&uuid, &winning) {
                            Ok(w) => println!("{} resolved as {} (previous: {})", uuid, w, winning),
                            Err(e) => {
                                eprintln!(
                                    "{} failed to resolve as {}: {}",
                                    uuid,
                                    winning,
                                    e.to_string()
                                );
                                exit(3);
                            }
                        };
                    }
                } else {
                    // Resolve all conflicts
                    let in_conflict = m.in_conflict();
                    for uuid in &in_conflict {
                        let winning = m.get_winner(uuid).unwrap();
                        match m.resolve_as(uuid, &winning) {
                            Ok(w) => println!("{} resolved as {} (previous: {})", uuid, w, winning),
                            Err(e) => {
                                eprintln!(
                                    "{} failed to resolve as {}: {}",
                                    uuid,
                                    winning,
                                    e.to_string()
                                );
                                exit(3);
                            }
                        };
                    }
                }
                m.commit(None).expect("Failed to commit changes");
                println!("Committed changes");
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
        Commands::Value {
            source,
            username,
            password,
            object,
            revision,
        } => match Url::parse(&source.unwrap()) {
            Ok(url) => {
                // Resolve specific uuid
                let adapter = get_adapter(url, username, password);
                let m =
                    Melda::new(Arc::new(RwLock::new(adapter))).expect("Failed to inizialize Melda");
                match revision {
                    Some(r) => match m.get_value(&object, &r) {
                        Ok(v) => {
                            println!("{}", serde_json::to_string(&v).unwrap());
                        }
                        Err(e) => {
                            eprintln!("Invalid object or revision {} {}: {}", object, r, e);
                            exit(3);
                        }
                    },
                    None => match m.get_winner(&object) {
                        Ok(r) => match m.get_value(&object, &r) {
                            Ok(v) => {
                                println!("{}", serde_json::to_string(&v).unwrap());
                            }
                            Err(e) => {
                                eprintln!("Invalid object or revision {} {}: {}", object, r, e);
                                exit(3);
                            }
                        },
                        Err(e) => {
                            eprintln!("Invalid object {}: {}", object, e);
                            exit(3);
                        }
                    },
                }
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
        Commands::History {
            source,
            username,
            password,
            object,
        } => match Url::parse(&source.unwrap()) {
            Ok(url) => {
                // Resolve specific uuid
                let adapter = get_adapter(url, username, password);
                let m =
                    Melda::new(Arc::new(RwLock::new(adapter))).expect("Failed to inizialize Melda");
                match m.get_winner(&object) {
                    Ok(w) => {
                        let mut crev = Some(w);
                        while crev.is_some() {
                            println!("{}", crev.as_ref().unwrap().to_string());
                            crev = m.get_parent_revision(&object, &crev.unwrap()).unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Invalid object {}: {}", object, e);
                        exit(3);
                    }
                }
            }
            _ => {
                eprintln!("Invalid Url");
            }
        },
    }
}
