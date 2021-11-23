use clap::*;
use rand::prelude::SliceRandom;
use std::fs;
use std::path::Path;
use svg2polylines::*;
use rand::*;

fn main() {
    let matches = App::new("svg_to_ahk")
        .author("Perondas")
        .arg(
            Arg::with_name("source")
                .short("s")
                .takes_value(true)
                .value_name("SOURCE FILE"),
        )
        .arg(
            Arg::with_name("destination")
                .short("d")
                .takes_value(true)
                .value_name("DESTINATION PATH"),
        )
        .arg(
            Arg::with_name("scale")
                .short("c")
                .takes_value(true)
                .value_name("SCALE"),
        )
        .arg(
            Arg::with_name("grain")
                .short("g")
                .takes_value(true)
                .value_name("GRAIN LEVEL"),
        )
        .arg(
            Arg::with_name("xOffset")
                .short("x")
                .takes_value(true)
                .value_name("X OFFSET"),
        )
        .arg(
            Arg::with_name("yOffset")
                .short("y")
                .takes_value(true)
                .value_name("Y OFFSET"),
        )
        .arg(
            Arg::with_name("pause")
                .short("p")
                .takes_value(true)
                .value_name("PAUSE DURATION"),
        )
        .arg(
            Arg::with_name("startingInterval")
                .short("i")
                .takes_value(true)
                .value_name("STARTING INTERVAL"),
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
            println!("No source file provided");
            return;
        }
    }

    let s_path = Path::new(source);

    if !s_path.exists() || !s_path.is_file() {
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
            println!("Failed to read file");
            println!("{}", m);
            return;
        }
    };

    let mut polygons = match parse(&content, grain) {
        Ok(p) => p,
        Err(m) => {
            println!("Failed to parse svg");
            println!("{}", m);
            return;
        }
    };

    let mut code =
        String::from("#SingleInstance Force\nCoordMode, Mouse, Screen\nSetDefaultMouseSpeed, 1\n");
    code.push_str("Escape::\nExitApp\nReturn\n^b::Box()\n");
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

    code.push_str(&format!(
        "Box() {{\n    Sleep, {}\n    Send {{Ctrl down}}\n",
        interval
    ));
    code.push_str(&format!("    MouseMove, {x}, {y}\n", x = min_x, y = min_y));
    code.push_str("    Sleep 200\n");

    code.push_str("    Send {lbutton down}\n");

    code.push_str(&format!("    MouseMove, {x}, {y}\n", x = min_x, y = max_y));
    code.push_str("    Sleep 200\n");

    code.push_str(&format!("    MouseMove, {x}, {y}\n", x = max_x, y = max_y));
    code.push_str("    Sleep 200\n");

    code.push_str(&format!("    MouseMove, {x}, {y}\n", x = max_x, y = min_y));
    code.push_str("    Sleep 200\n");

    code.push_str(&format!("    MouseMove, {x}, {y}\n", x = min_x, y = min_y));
    code.push_str("    Sleep 200\n");

    code.push_str("    Send {Ctrl up}\n    Send {lbutton up}\n    return\n}\n");

    code.push_str("^z::\nToggle := !Toggle\n");
    code.push_str(&format!(
        "Loop\n{{\n    If (!Toggle)\n        Break\n    Sleep, {}\n",
        interval
    ));

    let mut pause_time = 0;
    let mut draw_time = 0;

    let mut last_x = 0.0;
    let mut last_y = 0.0;


    polygons.shuffle(&mut thread_rng());

    for poly in polygons {
        if poly.len() < 10
            && ((poly[0].x - poly[poly.len() - 1].x).abs()
                + (poly[0].y - poly[poly.len() - 1].y).abs())
                < 20.0
        {
            continue;
        }

        let mut line = String::from("    Send {Ctrl down}\n");

        let mut md = false;
        for l in poly {
            let x = (l.x * scale) + x_offset;
            let y = (l.y * scale) + y_offset;
            line.push_str(&format!("    MouseMove, {x}, {y}\n", x = x, y = y));
            draw_time += 15;
            if !md {
                if ((last_x - x).abs() + (last_y - y).abs()) < 50.0 {
                    line.push_str(&format!("    Sleep, {}\n", pause));
                    pause_time += pause;
                }

                line.push_str("    Send {lbutton down}\n");
                md = true;
            }
            last_x = x;
            last_y = y;
        }
        draw_time += 50;
        line.push_str("    Send {Ctrl up}\n");
        line.push_str("    Send {LButton up}\n");
        line.push_str("    Sleep, 50\n");
        line.push('\n');
        code.push_str(&line);
    }

    code.push_str("    Send {lbutton up}\n");
    code.push_str("    Exit, 0\n");
    code.push_str("}\n");

    fs::write(destination.clone(), code).unwrap();

    println!("Created file {}", destination);
    println!("Total pause time: {}ms", pause_time);
    println!("Total expected time: {}ms", pause_time + draw_time);
    print!("Canvas of size: ");
    print!("x: {}, ", max_x - min_x);
    println!("y: {}", max_y - min_y);
}
