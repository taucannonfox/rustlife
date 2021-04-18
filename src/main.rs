extern crate olc_pixel_game_engine;
use crate::olc_pixel_game_engine as olc;

use std::time::SystemTime;

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
            self.game = GameOfLife::default();
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

impl Default for GameOfLife {
    fn default() -> Self {
        let screen_width_usize = SCREEN_WIDTH as usize;
        let screen_height_usize = SCREEN_HEIGHT as usize;
        let mut initial_state = vec![vec![false; screen_height_usize]; screen_width_usize];
        for y in 0..screen_height_usize {
            for x in 0..screen_width_usize {
                // Randomly set each cell to true or false
                initial_state[x][y] = (olc::c_rand() % 2) != 0;
            }
        }

        //print!("Initial state: {:?}", initial_state);

        return GameOfLife {
            state: initial_state,
            live_threshold: 3,
            die_threshold_lower: 2,
            die_threshold_upper: 3,
        };
    }
}

impl GameOfLife {
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



fn main() {
    // Set the RNG seed
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(time) => olc::c_srand(time.as_secs() as u32),
        Err(_)   => panic!("SystemTime is before UNIX Epoch!")
    }

    // Start the application
    let mut application = Application::new(GameOfLife::default());
    olc::start_with_full_screen_and_vsync(
        "RustLife",
        &mut application,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        SCREEN_SCALE,
        SCREEN_SCALE,
        false,
        true
    ).unwrap();
}
