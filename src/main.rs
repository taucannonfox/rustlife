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

        // Toggle step mode
        if olc::get_key(KEY_STEP_TOGGLE).pressed {
            self.step = !self.step;
            self.update_counter = 0.0;
        }

        // Reset the game state
        if olc::get_key(KEY_RESET).pressed {
            self.game = GameOfLife::new(olc::screen_width() as usize,
                                        olc::screen_height() as usize);
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
    live_threshold: u8,
    die_threshold_lower: u8,
    die_threshold_upper: u8,
}

impl GameOfLife {
    // Create a new game structure with a given width and height
    fn new(width: usize, height: usize) -> Self {
        let mut initial_state = vec![vec![false; height]; width];
        for y in 0..height {
            for x in 0..width {
                // Randomly set each cell to true or false
                initial_state[x][y] = rand::random();
            }
        }

        return GameOfLife {
            state: initial_state,
            live_threshold: 3,
            die_threshold_lower: 2,
            die_threshold_upper: 3,
        };
    }

    // Update the game state
    fn update(&mut self) {
        let screen_width_usize = olc::screen_width() as usize;
        let screen_height_usize = olc::screen_height() as usize;
        let mut new_state = self.state.clone();

        for y in 0..screen_height_usize {
            for x in 0..screen_width_usize {
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
        for y in 0..olc::screen_height() {
            for x in 0..olc::screen_width() {
                if self.state[x as usize][y as usize] {
                    olc::draw(x, y, olc::WHITE);
                }
            }
        }
    }

    // Get the number of living neighbors of the specified cell
    fn cell_get_neighbors(&self, x: i32, y: i32) -> u8 {
        let mut total = 0;
        for yofs in -1..=1 {
            for xofs in -1..=1 {
                if (0..olc::screen_width()).contains(&(x + xofs))          // x bounds check
                        && (0..olc::screen_height()).contains(&(y + yofs)) // y bounds check
                        && (xofs != 0 || yofs != 0) // Don't count center cell
                        && self.state[(x + xofs) as usize][(y + yofs) as usize] {
                    total += 1;
                }
            }
        }
        return total;
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
        .get_matches();

    let screen_width  = parse_arg(&args, "width",  SCREEN_WIDTH);
    let screen_height = parse_arg(&args, "height", SCREEN_HEIGHT);
    let screen_scale  = parse_arg(&args, "scale",  SCREEN_SCALE);

    // Start the application
    let game = GameOfLife::new(screen_width as usize, screen_height as usize);
    let mut application = Application::new(game);
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
