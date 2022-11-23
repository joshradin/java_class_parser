use clap::Parser;
use java_class_parser::error::Error;
use java_class_parser::{JavaClass, JavaClassParser};
use java_classpaths::Classpath;
use std::io::{stderr, stdin, stdout, Write};

#[derive(Debug, Parser)]
struct CliArgs {
    /// The classpath used to parse classes
    classpath: Classpath,
}

fn main() -> Result<(), Error> {
    let args: CliArgs = CliArgs::parse();
    println!("classpath: {}", args.classpath);
    println!();
    println!("Discover information about a class by typing it's fully qualified name. Specific information");
    println!("about it's methods or fields and be discovered by appending :methods or :fields to the name.");
    println!("You can exit this program by typing either 'quit' or 'exit'");

    let parser = JavaClassParser::with_classpath(args.classpath);
    let mut lines = stdin().lines();
    loop {
        print!("> ");
        stdout().flush()?;
        let line = if let Some(line) = lines.next() {
            line?
        } else {
            break;
        };
        let line = line.replace(".", "/");
        if line == "quit" || line == "exit" {
            break;
        }
        let (class, target) = match line.split_once(":") {
            Some((left, right)) => {
                (left, Some(right))
            },
            None => (&*line, None)
        };

        match parser.find(class) {
            Ok(class) => {
                match target {
                    None =>{
                        println!("{:#?}", class);
                    }
                    Some("methods") => {
                        let methods = class.methods();
                        for method in methods {
                            println!("{}: {}", method.name(), method.signature());
                        }
                    }
                    Some("fields") => {
                        let fields = class.fields();
                        for field in fields {
                            println!("{}: {}", field.name(), field.signature());
                        }
                    }
                    Some(_) => {
                        println!("only :methods and :fields are supported");
                    }
                }

            }
            Err(error) => {
                println!("error: {}", error);
            }
        }
    }

    Ok(())
}
