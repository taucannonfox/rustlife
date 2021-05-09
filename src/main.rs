extern crate clap;
extern crate olc_pixel_game_engine;
extern crate rand;

use crate::olc_pixel_game_engine as olc;

// Screen constants
const SCREEN_WIDTH:  i32 = 200;
const SCREEN_HEIGHT: i32 = 200;
const SCREEN_SCALE:  i32 = 4;

// How long to wait between updates
const UPDATE_TIME: f32 = 1.0 / 15.0;  // 15 FPS

// Key bindings
const KEY_STEP:        olc::Key = olc::Key::S;
const KEY_STEP_TOGGLE: olc::Key = olc::Key::SPACE;
const KEY_RESET:       olc::Key = olc::Key::R;
const KEY_EMPTY:       olc::Key = olc::Key::E;

/* ##########################################
# The main application structure.           #
# Handles events and drawing to the screen. #
########################################## */
struct Application {
    game:           GameOfLife,
    update_counter: f32,
    update_delta:   f32,
    step:           bool,  // Whether program should run automatically or be manually stepped
}

impl Application {
    fn new(game: GameOfLife) -> Self {
        Application {
            game: game,
            update_counter: 0.0,
            update_delta: UPDATE_TIME,
            step: false,
        }
    }
}

impl olc::Application for Application {
    // Called on application creation and destruction respectively
    fn on_user_create(&mut self) -> Result<(), olc::Error> { Ok(()) }
    fn on_user_destroy(&mut self) -> Result<(), olc::Error> { Ok(()) }

    // Called every frame
    fn on_user_update(&mut self, elapsed_time: f32) -> Result<(), olc::Error> {
        // Handle frame advance
        if self.step {
            // Advance frame on keypress
            if olc::get_key(KEY_STEP).pressed {
                self.game.update();
            }
        } else {
            // Limit to defined updates per second
            self.update_counter += elapsed_time;
            if self.update_counter >= self.update_delta {
                self.game.update();
                self.update_counter = 0.0;
            }
        }

        // Input handling
        if olc::get_key(KEY_EMPTY).pressed {
            // Reset with empty state
            self.game.empty_state();
            self.step = true;
        } else if olc::get_key(KEY_RESET).pressed {
            // Reset with random state
            self.game.randomize_state();
        } else if olc::get_key(KEY_STEP_TOGGLE).pressed {
            // Toggle step mode
            self.step = !self.step;
            self.update_counter = 0.0;
        }

        // Click to toggle a cell
        if olc::get_mouse(0).pressed {
            let x = olc::get_mouse_x() as usize;
            let y = olc::get_mouse_y() as usize;
            self.game.state[x][y] = !self.game.state[x][y];
        }

        self.game.draw();
        return Ok(());
    }
}

/* ################################################
# Conway's Game of Life                           #
# Handles updating and drawing of the game state. #
################################################ */
struct GameOfLife {
    state: Vec<Vec<bool>>,
    state_width: usize,
    state_height: usize,
    live_threshold: u8,
    die_threshold_lower: u8,
    die_threshold_upper: u8,
}

impl GameOfLife {
    // Create a new game structure with a given width and height
    fn new(width: usize, height: usize) -> Self {
        return GameOfLife {
            state: vec![vec![false; height]; width],
            state_width: width,
            state_height: height,
            live_threshold: 3,
            die_threshold_lower: 2,
            die_threshold_upper: 3,
        };
    }

    // Update the game state
    fn update(&mut self) {
        let mut new_state = self.state.clone();
        for y in 0..self.state_height {
            for x in 0..self.state_width {
                let neighbors = self.cell_get_neighbors(x as i32, y as i32);
                if self.state[x][y]
                        && (neighbors < self.die_threshold_lower
                        || neighbors > self.die_threshold_upper) {
                    // Kill cell if above or below bounds
                    new_state[x][y] = false;
                } else if !self.state[x][y] && neighbors == self.live_threshold {
                    // Create cell if neighbors are exactly at threshold
                    new_state[x][y] = true;
                }
            }
        }

        self.state = new_state;
    }

    // Draw the game state to the screen
    fn draw(&self) {
        olc::clear(olc::BLACK);
        for y in 0..self.state_height {
            for x in 0..self.state_width {
                if self.state[x][y] {
                    olc::draw(x as i32, y as i32, olc::WHITE);
                }
            }
        }
    }

    // Get the number of living neighbors of the specified cell
    fn cell_get_neighbors(&self, x: i32, y: i32) -> u8 {
        let mut total = 0;
        for yofs in -1..=1 {
            for xofs in -1..=1 {
                let x2 = (x + xofs) as usize;
                let y2 = (y + yofs) as usize;

                if (0..self.state_width).contains(&x2)          // x bounds check
                        && (0..self.state_height).contains(&y2) // y bounds check
                        && (xofs != 0 || yofs != 0) // Don't count center cell
                        && self.state[x2][y2] {
                    total += 1;
                }
            }
        }
        return total;
    }

    // Reset to an empty state
    fn empty_state(&mut self) {
        self.state = vec![vec![false; self.state_height]; self.state_width];
    }

    // Set each bit of the state randomly
    fn randomize_state(&mut self) {
        self.state = vec![vec![false; self.state_height]; self.state_width];
        for y in 0..self.state_height {
            for x in 0..self.state_width {
                // Randomly set each cell to true or false
                self.state[x][y] = rand::random();
            }
        }
    }
}


// Utility function to get a command line arg or return a default value
fn parse_arg<T: std::str::FromStr>(arg_matches: &clap::ArgMatches, arg: &str, default: T) -> T {
    if let Some(string) = &arg_matches.value_of(arg) {
        if let Ok(n) = string.parse::<T>() {
            return n;
        } else {
            eprintln!("ERROR: Couldn't parse value for argument `{}`", arg);
            std::process::exit(1);
        }
    } else {
        return default;
    }
}

fn main() {
    // Handle command line args
    let args = clap::App::new("RustLife")
        .version("0.1.0")
        .about("A Rust implementation of Conway's Game of Life")
        .arg(clap::Arg::with_name("width")
            .short("W")
            .long("width")
            .value_name("WIDTH")
            .help("Sets the simulation space's width")
            .takes_value(true))
        .arg(clap::Arg::with_name("height")
            .short("H")
            .long("height")
            .value_name("HEIGHT")
            .help("Sets the simulation space's height")
            .takes_value(true))
        .arg(clap::Arg::with_name("scale")
            .short("S")
            .long("scale")
            .value_name("SCALE")
            .help("Sets the display scale, i.e. how many pixels each cell should take up on the \
                screen")
            .takes_value(true))
        .arg(clap::Arg::with_name("start-paused")
            .long("start-paused")
            .help("Whether to start the simulation paused"))
        .get_matches();

    // Set screen parameters
    let screen_width  = parse_arg(&args, "width",  SCREEN_WIDTH);
    let screen_height = parse_arg(&args, "height", SCREEN_HEIGHT);
    let screen_scale  = parse_arg(&args, "scale",  SCREEN_SCALE);

    // Initialize the application
    let mut game = GameOfLife::new(screen_width as usize, screen_height as usize);
    game.randomize_state();
    let mut application = Application::new(game);

    // Start in step mode if specified on the command line
    if args.is_present("start-paused") {
        application.step = true;
    }

    // Start the application
    olc::start_with_full_screen_and_vsync(
        "RustLife",
        &mut application,
        screen_width,
        screen_height,
        screen_scale,
        screen_scale,
        false,
        true
    ).unwrap();
}
