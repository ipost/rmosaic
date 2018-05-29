extern crate clap;
use lib::params::clap::{App, Arg};

const DEFAULT_MAGNIFICATION: &'static str = "2";
const DEFAULT_PIXEL_GROUP_SIZE: &'static str = "16";
const DEFAULT_THREADS: &'static str = "2";

static mut PRINT_TIMING: bool = false;
static mut VERBOSITY: usize = 0;
static mut COLOR_CACHING: bool = false;

pub fn set_print_timings(p: bool) {
    unsafe { PRINT_TIMING = p }
}

pub fn print_timings() -> bool {
    unsafe { PRINT_TIMING }
}

pub fn set_color_caching(c: bool) {
    unsafe { COLOR_CACHING = c }
}

pub fn color_caching() -> bool {
    unsafe { COLOR_CACHING }
}

pub fn set_verbosity(v_level: usize) {
    unsafe { VERBOSITY = v_level }
}

pub fn verbosity(v_level: usize) -> bool {
    unsafe { VERBOSITY >= v_level }
}

pub fn get_parameters() -> clap::ArgMatches<'static> {
    App::new("Rust photomosaic builder")
        .version("0.1.0") // 🤔
        .author("Isaac Post <post.isaac@gmail.com>")
        .about("Makes photomosaics")
        .arg(
            Arg::with_name("INPUT")
            .help("Sets the input file which will be recreated as a mosaic")
            .required(true)
            .index(1),
            )
        .arg(
            Arg::with_name("LIBRARY")
            .help("The directory containing the images to be used as mosaic tiles")
            .required(true)
            .index(2),
            )
        .arg(
            Arg::with_name("OUT_FILE")
            .help("The name of the output mosaic image")
            .required(true)
            .index(3),
            )
        .arg(
            Arg::with_name("color-caching")
            .short("c")
            .long("color-caching")
            .help("Enables caching of closest-color matches. May improve performance if the input image has many identical colors repeated and/or you have a large image library."),
            )
        .arg(
            Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .multiple(true)
            .help("Sets the level of verbosity"),
            )
        .arg(
            Arg::with_name("print-timings")
            .short("t")
            .long("print-timings")
            .help("Print timings"),
            )
        .arg(
            Arg::with_name("SIZE")
            .short("g")
            .long("pixel-group-size")
            .help(&format!("The integer size of the square regions, in pixels, which will be replaced in the source image. Defaults to {}", DEFAULT_PIXEL_GROUP_SIZE))
            .takes_value(true)
            .required(false),
            )
        .arg(
            Arg::with_name("MAGNIFICATION_FACTOR")
            .short("m")
            .long("magnification")
            .help(&format!("The integer factor by which the original image's dimensions are increased. Defaults to {}", DEFAULT_MAGNIFICATION))
            .takes_value(true)
            .required(false),
            )
        .arg(
            Arg::with_name("THREADS")
            .long("threads")
            .help(&format!("The number of threads used. Defaults to {}", DEFAULT_THREADS))
            .takes_value(true)
            .required(false),
            )
        .get_matches()
}

pub fn parameters() -> (String, String, String, bool, bool, usize, u32, u32, usize) {
    let params = get_parameters();
    let source_image_path = params.value_of("INPUT").unwrap().to_string();
    let library_dir_path = params.value_of("LIBRARY").unwrap().to_string();
    let out_file_path = params.value_of("OUT_FILE").unwrap().to_string();
    let p_t = params.is_present("print-timings");
    let c_c = params.is_present("color-caching");
    let v = params.occurrences_of("verbose") as usize;
    let pixel_group_size = params
        .value_of("SIZE")
        .unwrap_or(DEFAULT_PIXEL_GROUP_SIZE)
        .parse::<u32>()
        .expect("Invalid value for SIZE");
    let magnification_factor = params
        .value_of("MAGNIFICATION_FACTOR")
        .unwrap_or(DEFAULT_MAGNIFICATION)
        .parse::<u32>()
        .expect("Invalid value for MAGNIFICATION_FACTOR");
    let threads = params
        .value_of("THREADS")
        .unwrap_or(DEFAULT_THREADS)
        .parse::<usize>()
        .expect("Invalid value for THREADS");
    (
        source_image_path,
        library_dir_path,
        out_file_path,
        p_t,
        c_c,
        v,
        pixel_group_size,
        magnification_factor,
        threads,
    )
}
