use std::process::ExitCode;


#[derive(Debug)]
struct ProgramOptions {
    input_filepath: Option<String>,
    output_filepath: Option<String>,
    allow_overwrite:bool,
    tolerance:f64,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            input_filepath: Option::None,
            output_filepath: Option::None,
            allow_overwrite: false,
            tolerance: 0.1
        }
    }
}

impl ProgramOptions {
    pub fn parse(&mut self, args:&[String]) -> Result<(), String> {
        if args.len() == 1 {
            // Special case.
            self.allow_overwrite = true;
            self.input_filepath = Some(args[0].clone());
            self.output_filepath = Some(format!("{}.stl", args[0]));
            return Result::Ok(());
        }

        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if arg == "--allow_overwrite" {
                self.allow_overwrite = true;
            } else if arg == "--tolerance" {
                let arg = iter.next().ok_or(String::from("Tolerance value not found."))?;
                let val = arg.parse::<f64>();
                if val.is_err() {
                    return Result::Err(format!("{}", val.err().unwrap()));
                }
                self.tolerance = val.unwrap();
                if self.tolerance <= 0.0 {
                    return Result::Err(String::from("specified tolerance is negative value."));
                }
            } else {
                if self.input_filepath.is_none() {
                    self.input_filepath = Option::Some(String::from(arg));
                } else if self.output_filepath.is_none() {
                    self.output_filepath = Option::Some(String::from(arg));
                } else {
                    return Result::Err(String::from("Too many filepath specified."));
                }
            }
        }

        return Result::Ok(());
    }
}


struct App {
    options: ProgramOptions
}

impl App {
    pub fn new() -> Self {
        App {
            options: Default::default()
        }
    }

    pub fn check_option(&self) -> Result<(), String> {
        if self.options.input_filepath.is_none() ||
                !std::fs::exists(self.options.input_filepath.as_ref().unwrap()).unwrap() {
            return Result::Err(String::from("Invalid input filepath specified."));
        }
        if self.options.output_filepath.is_none() {
            return Result::Err(String::from("Output filepath not specified."));
        }
        if !self.options.allow_overwrite && std::fs::exists(self.options.output_filepath.as_ref().unwrap()).unwrap() {
            return Result::Err(String::from("Output filepath already exists."));
        }

        return Result::Ok(());
    }

    pub fn load_step(&self) -> Result<cadrum::Mesh, String> {
        let mut source = std::fs::File::open(self.options.input_filepath.as_ref().unwrap()).unwrap();
        let solid = cadrum::read_step(&mut source)
            .map_err(|e| format!("{}", e))?;
        if solid.len() == 0 {
            return Result::Err("No solid loaded.".into());
        }
        let mesh = cadrum::mesh(&solid, self.options.tolerance)
            .map_err(|e| format!("{}", e))?;
        if mesh.indices.len() == 0 {
            return Result::Err("No mesh loaded.".into());
        }
        return Result::Ok(mesh);
    }

    pub fn export_stl(&self, mesh:&cadrum::Mesh) -> Result<(), String> {
        let mut stl = std::fs::File::create(self.options.output_filepath.as_ref().unwrap()).expect("create file");
        mesh.write_stl(&mut stl).map_err(|e| format!("{}", e))?;

        return Result::Ok(());
    }

    pub fn print_result(&self, mesh:&cadrum::Mesh) {
        let input_filesize = std::fs::metadata(self.options.input_filepath.as_ref().unwrap()).unwrap().len();
        let output_filesize = std::fs::metadata(self.options.output_filepath.as_ref().unwrap()).unwrap().len();
        let mesh_count = mesh.indices.len() / 3;
        println!("Input File Size: {} bytes", input_filesize);
        println!("Write to \"{}\"", self.options.output_filepath.as_ref().unwrap());
        println!("Output File Size: {} bytes", output_filesize);
        println!("Mesh count: {}", mesh_count);
    }
}


fn main() -> ExitCode{
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut app = App::new();
    let result = app.options.parse(&args);
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
        return ExitCode::from(1);
    }
    match app.check_option() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{}", err);
            return ExitCode::from(2);
        }
    }

    let mesh = app.load_step();
    if mesh.is_err() {
        eprintln!("{}", mesh.err().unwrap());
        return ExitCode::from(3);
    }
    let mesh= mesh.unwrap();

    let result = app.export_stl(&mesh);
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
        return ExitCode::from(4);
    }

    app.print_result(&mesh);
    
    return ExitCode::SUCCESS;
}
