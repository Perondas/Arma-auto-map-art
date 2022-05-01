use clap::*;
use std::fs;
use std::path::Path;
use std::time::Duration;
use svg2polylines::*;

fn main() {
    let matches = Command::new("Arma 3 auto map art generator")
        .author("Perondas <Pperondas@gmail.com>")
        .version("0.1.9")
        .about("Generates Ahk scripts from svg files to draw on the Arma 3 map screen")
        .arg(
            Arg::new("source")
                .short('s')
                .takes_value(true)
                .value_name("SOURCE FILE")
                .help("Path to the source svg file"),
        )
        .arg(
            Arg::new("destination")
                .short('d')
                .takes_value(true)
                .value_name("DESTINATION PATH")
                .help("Path to the destination file"),
        )
        .arg(
            Arg::new("scale")
                .short('c')
                .takes_value(true)
                .value_name("SCALE")
                .help("Sets the scale of the drawing")
                .default_value("1.0"),
        )
        .arg(
            Arg::new("grain")
                .short('g')
                .takes_value(true)
                .value_name("GRAIN LEVEL")
                .help("Sets how fine to draw. Higher values make it more course")
                .default_value("0.15"),
        )
        .arg(
            Arg::new("xOffset")
                .short('x')
                .takes_value(true)
                .value_name("X OFFSET")
                .help("Drawing offset on x axis")
                .default_value("0"),
        )
        .arg(
            Arg::new("yOffset")
                .short('y')
                .takes_value(true)
                .value_name("Y OFFSET")
                .help("Drawing offset on y axis")
                .default_value("0"),
        )
        .arg(
            Arg::new("pause")
                .short('p')
                .takes_value(true)
                .value_name("PAUSE DURATION")
                .help("Time to wait in between drawing 2 close lines to prevent marker dialogue")
                .default_value("750"),
        )
        .arg(
            Arg::new("startingInterval")
                .short('i')
                .takes_value(true)
                .value_name("STARTING INTERVAL")
                .help("The time between pressing a key and the drawing process starting")
                .default_value("3000"),
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .help("Outputs a test file"),
        )
        .arg(
            Arg::new("filter")
                .short('f')
                .long("filter")
                .help("Filters out small specks"),
        )
        .arg(
            Arg::new("on server")
                .long("onServer")
                .short('o')
                .help("Set this flag if you are going to be drawing on a server"),
        )
        .get_matches();

    let source;
    let mut destination;
    let scale;
    let grain;
    let x_offset;
    let y_offset;
    let pause;
    let interval;

    let server = matches.is_present("on server");

    let filter = matches.is_present("filter");

    let test = matches.is_present("test");

    interval = match matches.value_of("startingInterval") {
        Some(i) => match i.parse::<u32>() {
            Ok(v) => v,
            Err(m) => {
                println!("Could not parse starting interval");
                println!("{}", m);
                return;
            }
        },
        None => 3000,
    };

    pause = match matches.value_of("pause") {
        Some(p) => match p.parse::<u32>() {
            Ok(t) => t,
            Err(m) => {
                println!("Could not parse pause");
                println!("{}", m);
                return;
            }
        },
        None => 750,
    };

    match matches.value_of("source") {
        Some(s) => source = s,
        None => {
            if !test {
                println!("No source file provided");
                return;
            } else {
                source = "test.svg"
            }
        }
    }

    let s_path = Path::new(source);

    if !test && (!s_path.exists() || !s_path.is_file()) {
        println!("Could not find {}", source);
        return;
    }

    match matches.value_of("destination") {
        Some(d) => destination = String::from(d),
        None => match s_path.extension() {
            Some(ex) => {
                destination = String::from(s_path.to_str().unwrap());
                for _ in 0..ex.len() {
                    destination.pop();
                }
                destination.push_str("ahk");
            }
            None => {
                destination = String::from(s_path.to_str().unwrap());
                destination.push_str(".ahk");
            }
        },
    };

    scale = match matches.value_of("scale") {
        Some(s) => match s.parse::<f64>() {
            Ok(v) => v,
            Err(m) => {
                println!("Could not parse scale");
                println!("{}", m);
                return;
            }
        },
        None => 1.0,
    };

    grain = match matches.value_of("grain") {
        Some(s) => match s.parse::<f64>() {
            Ok(v) => v,
            Err(m) => {
                println!("Could not parse grain");
                println!("{}", m);
                return;
            }
        },
        None => 0.15,
    };

    x_offset = match matches.value_of("xOffset") {
        Some(s) => match s.parse::<f64>() {
            Ok(v) => v,
            Err(m) => {
                println!("Could not parse x offset");
                println!("{}", m);
                return;
            }
        },
        None => 1.0,
    };

    y_offset = match matches.value_of("yOffset") {
        Some(s) => match s.parse::<f64>() {
            Ok(v) => v,
            Err(m) => {
                println!("Could not parse y offset");
                println!("{}", m);
                return;
            }
        },
        None => 1.0,
    };

    let content = match fs::read_to_string(s_path) {
        Ok(c) => c,
        Err(m) => {
            if !test {
                println!("Failed to read file");
                println!("{}", m);
                return;
            }
            String::new()
        }
    };

    let mut polygons;

    if test {
        polygons = gen_test_polygons();
        println!("In test mode!");
    } else {
        // Parses the svg to lines
        // The parse function does not return a Err if the file was not a svg. It will return a empty vector.
        polygons = match parse(&content, grain) {
            Ok(p) => p,
            Err(m) => {
                println!("Failed to parse svg");
                println!("{}", m);
                return;
            }
        };

        // Filters out small lines
        let mut filtered_poly = Vec::new();
        for poly in polygons {
            if filter
                && poly.len() < 10
                && ((poly[0].x - poly[poly.len() - 1].x).abs()
                    + (poly[0].y - poly[poly.len() - 1].y).abs())
                    < 20.0
            {
                continue;
            }
            filtered_poly.push(poly);
        }

        // Sorts the polygons by their manhattan distance from the origin
        filtered_poly.sort_by(|a, b| {
            let a1 = a.last().unwrap();
            let b1 = b.first().unwrap();
            (a1.x + a1.y)
                .abs()
                .partial_cmp(&(b1.x + b1.y).abs())
                .unwrap()
        });

        polygons = Vec::with_capacity(filtered_poly.len());

        // For n polygons at position kx they will be sorted k1, k1 + n/2, k2, k2 + n/2, ..., k(n/2) - 1, k(n/2) + n/2;
        // This makes sure that no 2 polygons are adjacent (hopefully)
        for _ in 0..filtered_poly.len() / 2 {
            let a = filtered_poly.remove(0);
            let b = filtered_poly.remove(filtered_poly.len() / 2);
            polygons.push(a);
            polygons.push(b);
        }
        if !filtered_poly.is_empty() {
            polygons.push(filtered_poly.remove(0));
        }
    }

    let line_count = polygons.len();

    if line_count == 0 {
        // If vector empty, then the file must have had a issue. Probably not a svg file.
        println!("Failed to parse file! Please make sure to select a valid svg file!");
        return;
    }

    let longest_line = polygons.iter().map(|p| p.len()).max().unwrap();

    let mut code = AhkCode::new(true, 1);

    // Adds a exit keybind to the escape key
    let mut code = code.add_exit();

    // Finds the max and min points in x and y

    let max_x = polygons
        .iter()
        .flatten()
        .map(|p| (p.x * scale) + x_offset)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let max_y = polygons
        .iter()
        .flatten()
        .map(|p| (p.y * scale) + y_offset)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let min_y = polygons
        .iter()
        .flatten()
        .map(|p| (p.y * scale) + y_offset)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let min_x = polygons
        .iter()
        .flatten()
        .map(|p| (p.x * scale) + x_offset)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    // Adds a bounding box to the ahk file
    code = code.add_box(interval, (min_x, min_y), (max_x, max_y));

    // Adds the basic structure of the ahk script
    code.code.push(CodeEmital::EmitLine("^z::".to_string()));
    code.code
        .push(CodeEmital::EmitLine("Toggle := !Toggle".to_string()));
    code.code.push(CodeEmital::EmitLine("Loop".to_string()));
    code.code.push(CodeEmital::EmitLine("{".to_string()));
    code.code.push(CodeEmital::TabIn);
    code.code
        .push(CodeEmital::EmitLine("If (!Toggle)".to_string()));
    code.code.push(CodeEmital::TabIn);
    code.code.push(CodeEmital::EmitLine("Break".to_string()));
    code.code.push(CodeEmital::TabOut);
    code.code.push(CodeEmital::Sleep(interval));

    let mut last_x = 0.0;
    let mut last_y = 0.0;

    let mut point_count = 0;

    for poly in polygons {
        // For every polygon press Ctrl
        point_count += poly.len();
        code.code.push(CodeEmital::CtrlDown);

        // Mouse down flag
        let mut md = false;
        for l in poly {
            // Calculate the absolute position of the target coordinates
            let x = (l.x * scale) + x_offset;
            let y = (l.y * scale) + y_offset;
            // Move there
            code.code.push(CodeEmital::MouseMove(x, y));

            if !md {
                // Mouse down on the first line
                code.code.push(CodeEmital::Sleep(40));
                if (last_x - x).abs() + (last_y - y).abs() < 50.0 {
                    // Pause to avoid opening the marker dialog
                    code.code.push(CodeEmital::Sleep(pause));
                }
                if server {
                    // Slowly put down mouse to avoid drawing unwanted lines
                    code.code.push(CodeEmital::Sleep(150));
                    code.code.push(CodeEmital::MouseDown);
                    code.code.push(CodeEmital::Sleep(150));
                } else {
                    code.code.push(CodeEmital::MouseDown);
                }

                md = true;
            }

            if (last_x - x).abs() + (last_y - y).abs() > 15.0 {
                // Slow down drawing of straight lines
                code.code.push(CodeEmital::Sleep(40));
            }

            last_x = x;
            last_y = y;
        }

        // End of polygon
        code.code.push(CodeEmital::CtrlUp);
        code.code.push(CodeEmital::MouseUp);
        // Wait between polygons
        if server {
            code.code.push(CodeEmital::Sleep(200));
        } else {
            code.code.push(CodeEmital::Sleep(40));
        }

        // Add a blank line for readability
        code.code.push(CodeEmital::EmitLine(String::new()));
    }

    // Kills the loop in the script once it is done, to avoid accidental re-starting
    code.code.push(CodeEmital::EmitLine("Exit, 0".to_string()));
    code.code.push(CodeEmital::TabOut);
    code.code.push(CodeEmital::EmitLine("}".to_string()));

    // Builds the script and calculates total time spend drawing and all pauses summed up
    let (script, draw_time, pause_time) = code.build();

    // Save the file
    match fs::write(destination.clone(), script) {
        Ok(()) => (),
        Err(e) => {
            println!("Failed to save file!");
            println!("Error: {}", e);
            return;
        }
    }

    // Print information about the saved file
    println!();
    println!("Created file {}", destination);
    println!("Total line count: {}", line_count);
    println!("Total point count: {}", point_count);
    println!("Longest line: {}", longest_line);
    println!("Total pause time: {} seconds", pause_time.as_secs_f64());
    println!("Total draw time: {:.2} seconds", (draw_time).as_secs_f64());
    println!(
        "Total expected time: {:.2} minutes",
        (pause_time + draw_time).as_secs_f64() / 60.0
    );
    println!(
        "Canvas of size: x:{x:.2}, y:{y:.2}",
        x = max_x - min_x,
        y = max_y - min_y
    );
}

enum CodeEmital {
    TabIn,
    TabOut,
    EmitLine(String),
    MouseMove(f64, f64),
    Sleep(u32),
    CtrlDown,
    CtrlUp,
    MouseDown,
    MouseUp,
}

struct AhkCode {
    code: Vec<CodeEmital>,
}

impl AhkCode {
    pub fn new(force_single: bool, default_speed: u32) -> AhkCode {
        let mut code = Vec::new();

        if force_single {
            code.push(CodeEmital::EmitLine("#SingleInstance Force".to_string()));
        }

        code.push(CodeEmital::EmitLine("CoordMode, Mouse, Screen".to_string()));

        if default_speed > 100 {
            panic!("Speed was out of range!");
        }

        code.push(CodeEmital::EmitLine(format!(
            "SetDefaultMouseSpeed, {}",
            default_speed
        )));

        AhkCode { code }
    }

    pub fn add_exit(&mut self) -> &mut AhkCode {
        self.code.push(CodeEmital::EmitLine("Escape::".to_string()));
        self.code.push(CodeEmital::EmitLine("ExitApp".to_string()));
        self.code.push(CodeEmital::EmitLine("Return".to_string()));
        self
    }

    pub fn add_box(&mut self, pause: u32, min: (f64, f64), max: (f64, f64)) -> &mut AhkCode {
        self.code
            .push(CodeEmital::EmitLine("^b::Box()".to_string()));
        self.code.push(CodeEmital::EmitLine("Box() {".to_string()));
        self.code.push(CodeEmital::TabIn);

        self.code.push(CodeEmital::Sleep(pause));
        self.code.push(CodeEmital::CtrlDown);

        self.code.push(CodeEmital::MouseMove(min.0, min.1));
        self.code.push(CodeEmital::Sleep(200));

        self.code.push(CodeEmital::MouseDown);

        self.code.push(CodeEmital::MouseMove(min.0, max.1));
        self.code.push(CodeEmital::Sleep(200));

        self.code.push(CodeEmital::MouseMove(max.0, max.1));
        self.code.push(CodeEmital::Sleep(200));

        self.code.push(CodeEmital::MouseMove(max.0, min.1));
        self.code.push(CodeEmital::Sleep(200));

        self.code.push(CodeEmital::MouseMove(min.0, min.1));
        self.code.push(CodeEmital::Sleep(200));

        self.code.push(CodeEmital::CtrlUp);
        self.code.push(CodeEmital::MouseUp);
        self.code.push(CodeEmital::EmitLine("return".to_string()));
        self.code.push(CodeEmital::TabOut);

        self.code.push(CodeEmital::EmitLine("}".to_string()));

        self
    }

    pub fn build(&self) -> (String, Duration, Duration) {
        let mut spaces = String::new();
        let mut code = String::new();
        let mut pause_time = Duration::new(0, 0);
        let mut draw_time = Duration::new(0, 0);
        for line in &self.code {
            match line {
                CodeEmital::TabIn => spaces.push_str("    "),
                CodeEmital::TabOut => {
                    for _ in 0..4 {
                        spaces.pop();
                    }
                }
                CodeEmital::EmitLine(s) => code.push_str(&format!("{p}{s}\n", s = s, p = spaces)),
                CodeEmital::MouseMove(x, y) => {
                    code.push_str(&format!(
                        "{p}MouseMove, {x}, {y}\n",
                        p = spaces,
                        x = x,
                        y = y
                    ));
                    draw_time += Duration::from_millis(30);
                }
                CodeEmital::Sleep(t) => {
                    code.push_str(&format!("{p}Sleep, {t}\n", t = t, p = spaces));
                    pause_time += Duration::from_millis(*t as u64);
                }
                CodeEmital::CtrlDown => {
                    code.push_str(&format!("{p}Send {{Ctrl down}}\n", p = spaces))
                }
                CodeEmital::CtrlUp => code.push_str(&format!("{p}Send {{Ctrl up}}\n", p = spaces)),
                CodeEmital::MouseUp => {
                    code.push_str(&format!("{p}Send {{lbutton up}}\n", p = spaces))
                }
                CodeEmital::MouseDown => {
                    code.push_str(&format!("{p}Send {{lbutton down}}\n", p = spaces))
                }
            }
        }
        (code, draw_time, pause_time)
    }
}

fn gen_test_polygons() -> Vec<Vec<CoordinatePair>> {
    let mut result = Vec::new();
    for i in 1..50 {
        let mut line = Vec::new();

        let start = CoordinatePair {
            x: (i - 1 + i) as f64 / 2.0,
            y: 0.0,
        };

        let end = CoordinatePair {
            x: (i - 1 + i) as f64 / 2.0,
            y: 1.0,
        };
        line.push(start);
        line.push(end);
        result.push(line);
    }

    result
}
