use ink::graph::CommitGraph;
use ink::InkError;
use std::convert::TryInto;
use std::env;
use std::error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn error::Error>> {
    debugging_cli(std::env::args().collect())
}

fn debugging_cli(args: Vec<String>) -> Result<(), Box<dyn error::Error>> {
    if args.len() < 2 {
        return Err("No args provided".into());
    }

    match args[1].as_str() {
        "init" => ink::init(&env::current_dir()?.canonicalize()?)?,
        "commit" => {
            let _ = ink::commit()?;
        }
        "debug" => {
            if args.len() < 3 {
                return Err("Not enough args (commit, graph)".into());
            }

            match args[2].as_str() {
                "commit" => {
                    if args.len() < 4 {
                        return Err("Not enough args - commit hash".into());
                    }

                    let root_dir = root_dir()?.ok_or("no root")?;
                    let hash = hex::decode(&args[3])?.try_into().unwrap();
                    println!("{:?}", ink::commit::Commit::from(&hash, &root_dir));
                }
                "graph" => {
                    let root_dir = root_dir()?.ok_or("no root")?;
                    let graph = CommitGraph::get(&root_dir);
                    println!("{:?}", graph);
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    };

    Ok(())
}

fn root_dir() -> Result<Option<PathBuf>, InkError> {
    let curr_dir = env::current_dir()?.canonicalize()?;

    for path in curr_dir.ancestors() {
        let ink_dir = path.join(".ink");
        if ink_dir.exists() && ink_dir.is_dir() {
            return Ok(Some(ink_dir));
        }
    }

    Ok(None)
}
