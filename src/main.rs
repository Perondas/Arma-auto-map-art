use clap::*;
use std::fs;
use std::path::Path;
use svg2polylines::*;

fn main() {
    let matches = App::new("Arma 3 auto map art generator")
        .author("Perondas <Pperondas@gmail.com>")
        .version("0.1.3")
        .about("Generates Ahk scripts from svg files to draw on the Arma 3 map screen")
        .arg(
            Arg::with_name("source")
                .short("s")
                .takes_value(true)
                .value_name("SOURCE FILE")
                .help("Path to the source svg file"),
        )
        .arg(
            Arg::with_name("destination")
                .short("d")
                .takes_value(true)
                .value_name("DESTINATION PATH")
                .help("Path to the destination file"),
        )
        .arg(
            Arg::with_name("scale")
                .short("c")
                .takes_value(true)
                .value_name("SCALE")
                .help("Sets the scale of the drawing")
                .default_value("1.0"),
        )
        .arg(
            Arg::with_name("grain")
                .short("g")
                .takes_value(true)
                .value_name("GRAIN LEVEL")
                .help("Sets how fine to draw. Higher values make it more course")
                .default_value("0.15"),
        )
        .arg(
            Arg::with_name("xOffset")
                .short("x")
                .takes_value(true)
                .value_name("X OFFSET")
                .help("Drawing offset on x axis")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("yOffset")
                .short("y")
                .takes_value(true)
                .value_name("Y OFFSET")
                .help("Drawing offset on y axis")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("pause")
                .short("p")
                .takes_value(true)
                .value_name("PAUSE DURATION")
                .help("Time to wait in between drawing 2 close lines to prevent marker dialogue")
                .default_value("750"),
        )
        .arg(
            Arg::with_name("startingInterval")
                .short("i")
                .takes_value(true)
                .value_name("STARTING INTERVAL")
                .help("The time between pressing a key and the drawing process starting")
                .default_value("3000"),
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .help("Outputs a test file"),
        )
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("filter")
                .help("Filters out small specks"),
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
    let filter;
    let test;

    filter = matches.is_present("filter");

    test = matches.is_present("test");

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
        polygons = match parse(&content, grain) {
            Ok(p) => p,
            Err(m) => {
                println!("Failed to parse svg");
                println!("{}", m);
                return;
            }
        };

        let mut c = polygons.clone();
        c.sort_by(|a, b| {
            let a1 = a.last().unwrap();
            let b1 = b.first().unwrap();
            (a1.x + a1.y)
                .abs()
                .partial_cmp(&(b1.x + b1.y).abs())
                .unwrap()
        });
        let mut p = Vec::new();
        for _ in 0..c.len() / 2 {
            let a = c.remove(0);
            let b = c.remove(c.len() / 2);
            p.push(a);
            p.push(b);
        }
        if !c.is_empty() {
            polygons.push(c.remove(0));
        }
    }

    let mut code = AhkCode::new(true, 1);
    let mut code = code.add_exit();

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

    code = code.add_box(interval, (min_x, min_y), (max_x, max_y));

    code.code.push(CodeEmmitals::EmmitLine("^z::".to_string()));
    code.code
        .push(CodeEmmitals::EmmitLine("Toggle := !Toggle".to_string()));
    code.code.push(CodeEmmitals::EmmitLine("Loop".to_string()));
    code.code.push(CodeEmmitals::EmmitLine("{".to_string()));
    code.code.push(CodeEmmitals::TabIn);
    code.code
        .push(CodeEmmitals::EmmitLine("If (!Toggle)".to_string()));
    code.code.push(CodeEmmitals::TabIn);
    code.code.push(CodeEmmitals::EmmitLine("Break".to_string()));
    code.code.push(CodeEmmitals::TabOut);
    code.code.push(CodeEmmitals::Sleep(interval));

    let mut pause_time = 0;
    let mut draw_time = 0;

    let mut last_x = 0.0;
    let mut last_y = 0.0;

    for poly in polygons {
        if filter
            && poly.len() < 10
            && ((poly[0].x - poly[poly.len() - 1].x).abs()
                + (poly[0].y - poly[poly.len() - 1].y).abs())
                < 20.0
        {
            continue;
        }
        code.code.push(CodeEmmitals::CtrlDown);

        let mut md = false;
        for l in poly {
            let x = (l.x * scale) + x_offset;
            let y = (l.y * scale) + y_offset;
            code.code.push(CodeEmmitals::MouseMove(x, y));
            draw_time += 15;
            if !md {
                if (last_x - x).abs() + (last_y - y).abs() < 250.0 {
                    code.code.push(CodeEmmitals::Sleep(pause));
                    pause_time += pause;
                }

                code.code.push(CodeEmmitals::MouseDown);
                md = true;
            }
            last_x = x;
            last_y = y;
        }
        draw_time += 120;
        code.code.push(CodeEmmitals::CtrlUp);
        code.code.push(CodeEmmitals::MouseUp);
        code.code.push(CodeEmmitals::Sleep(120));
        code.code.push(CodeEmmitals::EmmitLine(String::new()));
    }

    code.code.push(CodeEmmitals::MouseUp);
    code.code
        .push(CodeEmmitals::EmmitLine("Exit, 0".to_string()));
    code.code.push(CodeEmmitals::TabOut);
    code.code.push(CodeEmmitals::EmmitLine("}".to_string()));

    let script = code.build();

    match fs::write(destination.clone(), script) {
        Ok(()) => (),
        Err(e) => {
            println!("Failed to save file!");
            println!("Error: {}", e);
            return;
        }
    }

    println!("Created file {}", destination);
    println!("Total pause time: {}ms", pause_time);
    println!("Total expected time: {}ms", pause_time + draw_time);
    print!("Canvas of size: ");
    print!("x: {}, ", max_x - min_x);
    println!("y: {}", max_y - min_y);
}

enum CodeEmmitals {
    TabIn,
    TabOut,
    EmmitLine(String),
    MouseMove(f64, f64),
    Sleep(u32),
    CtrlDown,
    CtrlUp,
    MouseDown,
    MouseUp,
}

struct AhkCode {
    code: Vec<CodeEmmitals>,
}

impl AhkCode {
    pub fn new(force_single: bool, default_speed: i32) -> AhkCode {
        let mut code = Vec::new();

        if force_single {
            code.push(CodeEmmitals::EmmitLine("#SingleInstance Force".to_string()));
        }

        code.push(CodeEmmitals::EmmitLine(
            "CoordMode, Mouse, Screen".to_string(),
        ));

        if default_speed > 100 {
            panic!("Speed was out of range!");
        }

        code.push(CodeEmmitals::EmmitLine(format!(
            "SetDefaultMouseSpeed, {}",
            default_speed
        )));

        AhkCode { code }
    }

    pub fn add_exit(&mut self) -> &mut AhkCode {
        self.code
            .push(CodeEmmitals::EmmitLine("Escape::".to_string()));
        self.code
            .push(CodeEmmitals::EmmitLine("ExitApp".to_string()));
        self.code
            .push(CodeEmmitals::EmmitLine("Return".to_string()));
        self
    }

    pub fn add_box(&mut self, pause: u32, min: (f64, f64), max: (f64, f64)) -> &mut AhkCode {
        self.code
            .push(CodeEmmitals::EmmitLine("^b::Box()".to_string()));
        self.code
            .push(CodeEmmitals::EmmitLine("Box() {".to_string()));
        self.code.push(CodeEmmitals::TabIn);

        self.code.push(CodeEmmitals::Sleep(pause));
        self.code.push(CodeEmmitals::CtrlDown);

        self.code.push(CodeEmmitals::MouseMove(min.0, min.1));
        self.code.push(CodeEmmitals::Sleep(200));

        self.code.push(CodeEmmitals::MouseDown);

        self.code.push(CodeEmmitals::MouseMove(min.0, max.1));
        self.code.push(CodeEmmitals::Sleep(200));

        self.code.push(CodeEmmitals::MouseMove(max.0, max.1));
        self.code.push(CodeEmmitals::Sleep(200));

        self.code.push(CodeEmmitals::MouseMove(max.0, min.1));
        self.code.push(CodeEmmitals::Sleep(200));

        self.code.push(CodeEmmitals::MouseMove(min.0, min.1));
        self.code.push(CodeEmmitals::Sleep(200));

        self.code.push(CodeEmmitals::CtrlUp);
        self.code.push(CodeEmmitals::MouseUp);
        self.code
            .push(CodeEmmitals::EmmitLine("return".to_string()));
        self.code.push(CodeEmmitals::TabOut);

        self.code.push(CodeEmmitals::EmmitLine("}".to_string()));

        self
    }

    pub fn build(&self) -> String {
        let mut spaces = String::new();
        let mut code = String::new();
        for line in &self.code {
            match line {
                CodeEmmitals::TabIn => spaces.push_str("    "),
                CodeEmmitals::TabOut => {
                    for _ in 0..4 {
                        spaces.pop();
                    }
                }
                CodeEmmitals::EmmitLine(s) => {
                    code.push_str(&format!("{p}{s}\n", s = s, p = spaces))
                }
                CodeEmmitals::MouseMove(x, y) => code.push_str(&format!(
                    "{p}MouseMove, {x}, {y}\n",
                    p = spaces,
                    x = x,
                    y = y
                )),
                CodeEmmitals::Sleep(t) => {
                    code.push_str(&format!("{p}Sleep, {t}\n", t = t, p = spaces))
                }
                CodeEmmitals::CtrlDown => {
                    code.push_str(&format!("{p}Send {{Ctrl down}}\n", p = spaces))
                }
                CodeEmmitals::CtrlUp => {
                    code.push_str(&format!("{p}Send {{Ctrl up}}\n", p = spaces))
                }
                CodeEmmitals::MouseUp => {
                    code.push_str(&format!("{p}Send {{lbutton up}}\n", p = spaces))
                }
                CodeEmmitals::MouseDown => {
                    code.push_str(&format!("{p}Send {{lbutton down}}\n", p = spaces))
                }
            }
        }
        code
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
