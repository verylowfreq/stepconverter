#![windows_subsystem = "windows"]

use std::process::ExitCode;

use cadrum::Tessellation;

mod gui;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Stl,
    Glb,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "stl" => Result::Ok(OutputFormat::Stl),
            "glb" => Result::Ok(OutputFormat::Glb),
            _ => Result::Err(format!("Unknown output format \"{}\". Supported formats: stl, glb", s)),
        }
    }

    pub fn from_extension(filepath: &str) -> Result<Self, String> {
        let ext = std::path::Path::new(filepath)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("stl") => Result::Ok(OutputFormat::Stl),
            Some("glb") => Result::Ok(OutputFormat::Glb),
            _ => Result::Err(format!("Could not infer output format from \"{}\". Specify --format stl|glb.", filepath)),
        }
    }
}


#[derive(Debug)]
struct ProgramOptions {
    input_filepath: Option<String>,
    output_filepath: Option<String>,
    allow_overwrite:bool,
    tolerance:f64,
    format: Option<OutputFormat>,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            input_filepath: Option::None,
            output_filepath: Option::None,
            allow_overwrite: false,
            tolerance: 0.1,
            format: Option::None,
        }
    }
}

impl ProgramOptions {
    pub fn parse(&mut self, args:&[String]) -> Result<(), String> {
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
            } else if arg == "--format" {
                let arg = iter.next().ok_or(String::from("Output format value not found."))?;
                self.format = Some(OutputFormat::parse(arg)?);
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

    pub fn resolve_format(&self) -> Result<OutputFormat, String> {
        if let Some(format) = self.format {
            return Result::Ok(format);
        }
        return OutputFormat::from_extension(self.output_filepath.as_ref().unwrap());
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
        self.options.resolve_format()?;

        return Result::Ok(());
    }

    pub fn load_step(&self) -> Result<cadrum::Mesh, String> {
        let mut source = std::fs::File::open(self.options.input_filepath.as_ref().unwrap()).unwrap();
        let solid = cadrum::Solid::read_step(&mut source)
                .map_err(|e| format!("{}", e))?;
        if solid.len() == 0 {
            return Result::Err("No solid loaded.".into());
        }
        let opts = Tessellation {
            deflection_linear: self.options.tolerance,
            relative_linear: true,
            ..Default::default()
        };
        let mesh = cadrum::Solid::mesh(&solid, opts)
                .map_err(|e| format!("{}", e))?;
        if mesh.indices.len() == 0 {
            return Result::Err("No mesh loaded.".into());
        }
        return Result::Ok(mesh);
    }

    pub fn export(&self, mesh:&cadrum::Mesh, format: OutputFormat) -> Result<(), String> {
        let mut file = std::fs::File::create(self.options.output_filepath.as_ref().unwrap()).expect("create file");
        match format {
            OutputFormat::Stl => mesh.write_stl(&mut file).map_err(|e| format!("{}", e))?,
            OutputFormat::Glb => mesh.write_gltf_binary(&mut file).map_err(|e| format!("{}", e))?,
        }

        return Result::Ok(());
    }

    pub fn format_result(&self, mesh:&cadrum::Mesh) -> String {
        let input_filesize = std::fs::metadata(self.options.input_filepath.as_ref().unwrap()).unwrap().len();
        let output_filesize = std::fs::metadata(self.options.output_filepath.as_ref().unwrap()).unwrap().len();
        let mesh_count = mesh.indices.len() / 3;
        format!(
            "Input File Size: {} bytes\nWrite to \"{}\"\nOutput File Size: {} bytes\nMesh count: {}",
            input_filesize,
            self.options.output_filepath.as_ref().unwrap(),
            output_filesize,
            mesh_count
        )
    }

    pub fn print_result(&self, mesh:&cadrum::Mesh) {
        println!("{}", self.format_result(mesh));
    }
}


#[cfg(windows)]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS); }
}

#[cfg(not(windows))]
fn attach_parent_console() {}

fn main() -> ExitCode{
    let args = std::env::args().skip(1).collect::<Vec<String>>();

    let is_gui_mode = args.is_empty() || (args.len() == 1 && !args[0].starts_with("--"));
    if is_gui_mode {
        return gui::run(args.get(0).cloned());
    }
    attach_parent_console();

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
    println!("Loading STEP file: {}", app.options.input_filepath.clone().unwrap());
    let mesh = app.load_step();
    if mesh.is_err() {
        eprintln!("{}", mesh.err().unwrap());
        return ExitCode::from(3);
    }
    let mesh= mesh.unwrap();

    let format = app.options.resolve_format().unwrap();
    let format_label = match format {
        OutputFormat::Stl => "STL",
        OutputFormat::Glb => "GLB",
    };
    println!("Save {} file: {}", format_label, app.options.output_filepath.clone().unwrap());
    let result = app.export(&mesh, format);
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
        return ExitCode::from(4);
    }

    app.print_result(&mesh);
    
    return ExitCode::SUCCESS;
}
