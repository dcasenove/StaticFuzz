use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use std::fs;
use std::fs::{File, OpenOptions};
use std::error::Error;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "parser")]
struct Opt {

    /// SARIF input file
    #[structopt(parse(from_os_str))]
    input_path: PathBuf,

    /// Output file in txt format
    #[structopt(parse(from_os_str))]
    output_file: PathBuf,
    
    /// Code_flow parsing
    #[structopt(short = "f", long = "flow")]
    code_flow: bool,
}

/// Describes the structure of a SARIF report
#[derive(Deserialize, Serialize, Debug)]
struct Report {
    /// Required - SARIF reports must have a version - 2.1.0
    version: String,
    /// A SARIF log file contains an array of one or more runs
    runs: Vec<Run>,
}

/// Describes the structure of the run, a single invocation of the tool
#[derive(Deserialize, Serialize, Debug)]
struct Run {
    /// Required - The tool which carried out the run
    tool: Tool,
    /// Required - The results of the run
    results: Vec<Results>
}

/// Describes the results of the run
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Results {
    /// Required - Message string reporting the result
    message: Message,
    /// Required - List of location objects 
    locations: Vec<Location>,
    /// Required - Code Flow
    codeFlows: Vec<Flows>,
}

/// Describes the location property
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Location {
    /// Physical location of the articat
    physicalLocation: PhysicalLocation,
}

/// Describes the location property
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Flows {
    threadFlows: Vec<ThreadFlows>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct ThreadFlows {
    locations: Vec<ThreadLocations>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct ThreadLocations {
    location: Location,
}

/// Describes the physical location of the error
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct PhysicalLocation {
    artifactLocation: Artifact,
    region: Region,
}

/// Describes the physical artifcat 
#[derive(Deserialize, Serialize, Debug)]
struct Artifact {
    uri: String,
}

/// Describes the region in which the error is found
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Region {
    startLine: u32,
}

/// Describes the message property
#[derive(Deserialize, Serialize, Debug)]
struct Message {
    /// Required - Text describing the message property
    text: String,
}

/// Describes the tool which carried out the run
#[derive(Deserialize, Serialize, Debug)]
struct Tool {
    /// Required - Sub-property driver of the tool
    driver: Driver,
}

/// Describes the sub-property
#[derive(Deserialize, Serialize, Debug)]
struct Driver {
    /// Required - Name of the tool which produced the analysis
    name: String
}


fn main() {
    let opt = Opt::from_args();

    let input_path = opt.input_path.into_os_string().into_string().unwrap();
    let output_file = opt.output_file.into_os_string().into_string().unwrap();
    let input_exist = Path::new(&input_path);
    assert!(input_exist.exists(), "Path doesn't exist");

    if input_exist.is_dir() {
        match fs::read_dir(input_exist) {
            Err(why) => println!("! {:?}", why.kind()),
            Ok(paths) => for path in paths {
                handle_file(path.unwrap()
                                .path()
                                .display()
                                .to_string()
                                ,output_file.to_owned()
                                ,opt.code_flow)
            },
        }
    }
    else {
        handle_file(input_path, output_file, opt.code_flow);
    }
}

/// Reads sarif json format from file
fn read_from_file(file: File) -> Result<Report, Box<dyn Error>> {
    let reader = BufReader::new(file);
    let report = serde_json::from_reader(reader)?;

    Ok(report)
}

/// Write parsed result to file
fn write_to_file(report: Report, output_path: PathBuf, code_flow: bool)
    -> std::io::Result<()> {
    let mut output_file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .append(true)
                            .open(output_path).unwrap();
    
    // Output version in format: URI Line
    for r in report.runs {
        for res in r.results {
            if !code_flow {
                for loc in res.locations {
                    writeln!(&mut output_file, "{}:{}",
                        loc.physicalLocation.artifactLocation.uri,
                        loc.physicalLocation.region.startLine,
                        //res.message.text
                        ).unwrap();
                }
            }
            else {
                for flow in res.codeFlows {
                    for t_flow in flow.threadFlows {
                        for loc in t_flow.locations {
                            writeln!(&mut output_file, "{}:{}",
                                loc.location.physicalLocation.artifactLocation.uri,
                                loc.location.physicalLocation.region.startLine,
                                //res.message.text
                                ).unwrap();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_file(input_path: String, output_file: String, code_flow: bool) {
    let input_path = Path::new(&input_path);

    // Check if file is in sarif extension and open it
    if input_path.extension().unwrap() == "sarif" {

    let file = match File::open(&input_path) {
        Err(why) => panic!("couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };

    // Parse sarif format
    let report = read_from_file(file).unwrap();

    // Output format to txt file
    let mut output_path = PathBuf::from(input_path);
    output_path.set_file_name(output_file);
    output_path.set_extension("custom_targets.txt");

    write_to_file(report, output_path, code_flow).
         expect("Failed to write parser output to file");
    }
}
